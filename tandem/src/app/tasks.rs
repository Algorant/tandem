//! Shared Task lifecycle operations.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::support::{
    active_task_descendant_ids, append_event, checkpoint_boundary, current_timestamp,
    hierarchy_from_project as hierarchy_from_workspace, require_nonempty,
    resolve_parent_relationship, resolved_task_descendants, unresolved_blockers_in_hierarchy,
    validate_state, validate_task_document_against_hierarchy, workspace_deprecation_warnings,
};
use crate::app::Error;
use crate::project::write::{ensure_file_unchanged, read_file_snapshot};
use crate::project::{
    self, patch_accord_content, patch_frontmatter_content, patch_resolution_content,
    render_accord_block, replace_markdown_body, write_atomic, yaml_double_quote, CheckpointOutcome,
    ProjectHierarchy as HierarchyIndex, StoredDocument as Document, TandemProject,
};
use crate::protocol::accord::{status as accord_status, AccordRecord};
use crate::protocol::document::{
    is_absolute_reference_url, parse_field_values, validate_task_kind, EFFORTS, PRIORITIES,
};
use crate::protocol::hierarchy::{DocumentLocation, ParentRelationship};
use crate::protocol::ids::next_sequential_number as next_sequential_number_for_ids;
use crate::protocol::workflow::{
    ResolutionRecord, RESOLUTION_OUTCOME_CANCELED, RESOLUTION_OUTCOME_COMPLETED,
};

fn inline_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| yaml_double_quote(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn push_optional_line(lines: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(format!("{key}: {}", yaml_double_quote(value.trim())));
    }
}

fn push_array_line(lines: &mut Vec<String>, key: &str, values: &[String]) {
    if !values.is_empty() {
        lines.push(format!("{key}: {}", inline_array(values)));
    }
}

/// Renders the accord block for a new task through the one shared renderer,
/// so creation and transitions can never write different shapes.
fn render_accord_block_for_new_task(options: &AddOptions, now: &str) -> String {
    render_accord_block(&AccordRecord {
        status: "ready".to_string(),
        acceptance: options.acceptance.clone(),
        constraints: options.constraints.clone(),
        validations: options.validations.clone(),
        updated_at: now.to_string(),
        ..AccordRecord::default()
    })
}

fn push_array_line_indented(lines: &mut Vec<String>, key: &str, values: &[String]) {
    if !values.is_empty() {
        lines.push(format!("  {key}: {}", inline_array(values)));
    }
}

#[derive(Debug, Default)]
pub(crate) struct AddOptions {
    pub(crate) title: Option<String>,
    pub(crate) state: Option<String>,
    pub(crate) json: bool,
    pub(crate) description: Option<String>,
    pub(crate) acceptance: Vec<String>,
    pub(crate) constraints: Vec<String>,
    pub(crate) validations: Vec<String>,
    pub(crate) kind: Option<String>,
    pub(crate) priority: Option<String>,
    pub(crate) effort: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) assignee: Option<String>,
    pub(crate) due_date: Option<String>,
    pub(crate) parent: Option<String>,
    pub(crate) blockers: Vec<String>,
    pub(crate) references: Vec<String>,
    pub(crate) related_files: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct AddOutcome {
    pub(crate) id: String,
    pub(crate) state: String,
    pub(crate) title: String,
    pub(crate) kind: Option<String>,
    pub(crate) parent: Option<String>,
    pub(crate) parent_relationship: Option<ParentRelationship>,
    pub(crate) path: PathBuf,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct MoveOptions {
    pub(crate) id: String,
    pub(crate) state: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct UpdateOptions {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) priority: Option<String>,
    pub(crate) effort: Option<String>,
    pub(crate) assignee: Option<String>,
    pub(crate) due_date: Option<String>,
    pub(crate) parent: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) blockers: Vec<String>,
    pub(crate) references: Vec<String>,
    pub(crate) related_files: Vec<String>,
    pub(crate) acceptance: Vec<String>,
    pub(crate) constraints: Vec<String>,
    pub(crate) validations: Vec<String>,
    pub(crate) clear: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct UpdateChange {
    pub(crate) field: String,
    pub(crate) old: String,
    pub(crate) new: String,
}

#[derive(Debug)]
pub(crate) struct UpdateOutcome {
    pub(crate) id: String,
    pub(crate) path: PathBuf,
    pub(crate) changes: Vec<UpdateChange>,
    pub(crate) warnings: Vec<String>,
    pub(crate) parent_relationship: Option<ParentRelationship>,
}

#[derive(Debug, Default)]
pub(crate) struct CompleteOptions {
    pub(crate) id: String,
    pub(crate) note: Option<String>,
    pub(crate) reviewer: Option<String>,
}

#[derive(Debug)]
pub(crate) struct CompleteOutcome {
    pub(crate) id: String,
    pub(crate) board_path: PathBuf,
    pub(crate) log_path: PathBuf,
    pub(crate) warnings: Vec<String>,
    pub(crate) has_completion_warnings: bool,
    pub(crate) checkpoint: CheckpointOutcome,
}

#[derive(Debug, Default)]
pub(crate) struct CancelOptions {
    pub(crate) id: String,
    pub(crate) reason: Option<String>,
}

#[derive(Debug)]
pub(crate) struct CancelOutcome {
    pub(crate) id: String,
    pub(crate) reason: String,
    pub(crate) board_path: PathBuf,
    pub(crate) log_path: PathBuf,
    pub(crate) checkpoint: CheckpointOutcome,
}

/// Create a Task, Epic, or Subtask after canonical hierarchy validation.
pub(crate) fn add(workspace: &TandemProject, options: AddOptions) -> Result<AddOutcome, Error> {
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    let title =
        require_nonempty(options.title.as_deref(), "add requires --title <title>")?.to_string();
    let state = options.state.as_deref().unwrap_or("todo").to_string();
    validate_state(workspace, &state)?;
    validate_task_kind_option(options.kind.as_deref(), "add --kind")?;
    if options.acceptance.is_empty() {
        return Err(Error::usage(
            "add requires at least one --acceptance <text>",
        ));
    }
    validate_optional_vocabulary(
        options.priority.as_deref(),
        "add --priority",
        PRIORITIES,
        "priority",
    )?;
    validate_optional_vocabulary(options.effort.as_deref(), "add --effort", EFFORTS, "effort")?;
    let kind = options
        .kind
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if kind.as_deref() == Some("epic") && options.parent.is_some() {
        return Err(Error::user(
            "Validation failed: an Epic cannot have parentId; remove --parent or --kind epic",
        ));
    }
    let hierarchy = hierarchy_from_workspace(workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let parent_relationship = options
        .parent
        .as_deref()
        .map(|parent| resolve_parent_relationship(&hierarchy, "task", parent))
        .transpose()?;
    if let Some(parent) = options.parent.as_deref() {
        let parent_document = hierarchy
            .document(parent)
            .expect("resolve_parent_relationship validated the parent");
        if parent_document.location != DocumentLocation::Board {
            return Err(Error::user(format!(
                "Validation failed: cannot add a task under archived parent {parent}; parent must be on the Board"
            )));
        }
    }
    for blocker in &options.blockers {
        if hierarchy.document(blocker).is_none() {
            return Err(Error::user(format!(
                "Validation failed: blocker document not found: {blocker}"
            )));
        }
    }

    let mut warnings = Vec::new();
    for reference in &options.references {
        if is_absolute_reference_url(reference) {
            continue;
        }
        if !workspace.reference_target_exists(reference)? {
            warnings.push(format!("reference not found: {reference}"));
        }
    }

    let allocation_prefix = match (parent_relationship, options.parent.as_deref()) {
        (Some(ParentRelationship::Subtask), Some(parent)) => parent,
        _ => "task",
    };
    let now = current_timestamp();
    let last_allocated = next_sequential_number_for_ids(
        hierarchy.documents.values().map(|doc| doc.id()),
        allocation_prefix,
    );
    let created = project::write::create_new_sequential_document_after(
        workspace,
        allocation_prefix,
        last_allocated,
        |task_id| {
            let mut lines = vec![
                "---".to_string(),
                format!("id: {task_id}"),
                "type: task".to_string(),
            ];
            push_optional_line(&mut lines, "kind", kind.as_deref());
            lines.push(format!("title: {}", yaml_double_quote(&title)));
            lines.push(format!("state: {state}"));
            push_optional_line(&mut lines, "priority", options.priority.as_deref());
            push_optional_line(&mut lines, "effort", options.effort.as_deref());
            push_optional_line(&mut lines, "assignee", options.assignee.as_deref());
            push_optional_line(&mut lines, "dueDate", options.due_date.as_deref());
            push_optional_line(&mut lines, "parentId", options.parent.as_deref());
            push_array_line(&mut lines, "blockers", &options.blockers);
            push_array_line(&mut lines, "references", &options.references);
            push_array_line(&mut lines, "relatedFiles", &options.related_files);
            push_array_line(&mut lines, "tags", &options.tags);
            lines.push(
                render_accord_block_for_new_task(&options, &now)
                    .trim_end()
                    .to_string(),
            );
            lines.push(format!("createdAt: {}", yaml_double_quote(&now)));
            lines.push(format!("updatedAt: {}", yaml_double_quote(&now)));
            lines.push("---".to_string());
            lines.push(String::new());
            if let Some(description) = options.description.as_deref() {
                lines.push("## Description".to_string());
                lines.push(String::new());
                lines.push(description.to_string());
            }
            lines.push(String::new());
            lines.join("\n")
        },
    )?;
    append_event(workspace, "task.created", &created.id, &title)?;

    Ok(AddOutcome {
        id: created.id,
        state,
        title,
        kind,
        parent: options.parent,
        parent_relationship,
        path: created.path,
        warnings,
    })
}

#[derive(Debug)]
pub(crate) struct MoveTaskOutcome {
    pub(crate) id: String,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) changed: bool,
    pub(crate) path: PathBuf,
    pub(crate) accord_sync: Option<String>,
}

pub(crate) fn move_to_state(
    workspace: &TandemProject,
    id: &str,
    state: &str,
) -> Result<MoveTaskOutcome, Error> {
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    validate_state(workspace, state)?;

    let hierarchy = hierarchy_from_workspace(workspace)?;
    let doc = hierarchy
        .document(id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| Error::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(Error::user(format!(
            "Validation failed: only task documents can be moved in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;
    let doc_id = doc.id().to_string();
    let previous_state = doc.field("state").unwrap_or("-").to_string();
    if previous_state == state {
        return Ok(MoveTaskOutcome {
            id: doc_id,
            from: previous_state,
            to: state.to_string(),
            changed: false,
            path: doc.path,
            accord_sync: None,
        });
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut updates = BTreeMap::new();
    updates.insert("state".to_string(), state.to_string());
    updates.insert("updatedAt".to_string(), now.clone());
    let mut patched = patch_frontmatter_content(&content, &updates, &[])?;
    let mut synced_accord_event = None;
    let mut accord_sync = None;
    if state == "in-progress" && accord_status(&doc) == Some("ready") {
        let mut accord = AccordRecord::from_document(&doc, &now);
        accord.status = "claimed".to_string();
        if accord.claimed_at.is_none() {
            accord.claimed_at = Some(now.clone());
        }
        patched = patch_accord_content(&patched, &accord)?;
        synced_accord_event = Some("accord.claimed");
        accord_sync = Some("ready -> claimed".to_string());
    }
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        workspace,
        "task.moved",
        &doc_id,
        &format!("Moved {doc_id} from {previous_state} to {state}"),
    )?;
    if let Some(event_name) = synced_accord_event {
        append_event(
            workspace,
            event_name,
            &doc_id,
            &format!("Synchronized accord claim for {doc_id} after move"),
        )?;
    }

    Ok(MoveTaskOutcome {
        id: doc_id,
        from: previous_state,
        to: state.to_string(),
        changed: true,
        path: doc.path,
        accord_sync,
    })
}

pub(crate) fn update(
    workspace: &TandemProject,
    options: UpdateOptions,
) -> Result<UpdateOutcome, Error> {
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    let hierarchy = hierarchy_from_workspace(workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| Error::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(Error::user(format!(
            "Validation failed: only task documents can be updated in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;
    validate_update_options(&options, &hierarchy)?;

    hierarchy.validate_all_task_hierarchies()?;
    let old_role = hierarchy
        .task_role(&doc)?
        .expect("active task has a task role");
    let mut prospective = doc.clone();
    if let Some(kind) = options.kind.as_deref() {
        prospective
            .fields
            .insert("kind".to_string(), kind.to_string());
    }
    if let Some(parent) = options.parent.as_deref() {
        prospective
            .fields
            .insert("parentId".to_string(), parent.to_string());
    }
    let prospective_hierarchy = hierarchy.with_replacement(prospective.clone());
    let prospective_role = prospective_hierarchy
        .task_role(&prospective)?
        .expect("prospective task has a task role");
    if options.parent.is_some() && old_role != prospective_role {
        return Err(Error::user(format!(
            "Validation failed: reparenting {} would change its canonical role from {} to {}; IDs are immutable",
            doc.id(),
            old_role.as_str(),
            prospective_role.as_str()
        )));
    }
    prospective_hierarchy.validate_all_task_hierarchies()?;
    let parent_relationship = if options.parent.is_some() {
        prospective_hierarchy.relationship(&prospective)?
    } else {
        None
    };

    let mut warnings = Vec::new();
    for reference in &options.references {
        if is_absolute_reference_url(reference) {
            continue;
        }
        if !workspace.reference_target_exists(reference)? {
            warnings.push(format!("reference not found: {reference}"));
        }
    }

    let mut updates = BTreeMap::new();
    let mut changes = Vec::new();
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "title",
        options.title.as_deref(),
    )?;
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "kind",
        options.kind.as_deref(),
    )?;
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "priority",
        options.priority.as_deref(),
    )?;
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "effort",
        options.effort.as_deref(),
    )?;
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "dueDate",
        options.due_date.as_deref(),
    )?;
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "parentId",
        options.parent.as_deref(),
    )?;
    apply_list_replace_update(&mut updates, &mut changes, &doc, "tags", &options.tags);
    apply_list_replace_update(
        &mut updates,
        &mut changes,
        &doc,
        "blockers",
        &options.blockers,
    );
    apply_list_replace_update(
        &mut updates,
        &mut changes,
        &doc,
        "references",
        &options.references,
    );
    apply_list_replace_update(
        &mut updates,
        &mut changes,
        &doc,
        "relatedFiles",
        &options.related_files,
    );
    let clear_fields = options
        .clear
        .iter()
        .map(|field| match field.as_str() {
            "parent" => "parentId",
            "due-date" => "dueDate",
            "related-file" => "relatedFiles",
            "blocker" => "blockers",
            "tag" => "tags",
            other => other,
        })
        .collect::<Vec<_>>();
    let accord_updates = apply_accord_definition_update(&mut changes, &doc, &options)?;
    let clear_body = options.clear.iter().any(|field| field == "body");
    let replacement_body = options
        .body
        .as_deref()
        .filter(|body| doc.body.as_str() != *body)
        .or(clear_body.then_some(""));
    if replacement_body.is_some() {
        changes.push(UpdateChange {
            field: "body".to_string(),
            old: "<body>".to_string(),
            new: "<body>".to_string(),
        });
    }

    for field in &clear_fields {
        if doc.field(field).is_some() && !changes.iter().any(|change| change.field == *field) {
            changes.push(UpdateChange {
                field: (*field).to_string(),
                old: "<set>".to_string(),
                new: "<cleared>".to_string(),
            });
        }
    }
    let doc_id = doc.id().to_string();
    let path = doc.path.clone();
    if changes.is_empty() {
        return Ok(UpdateOutcome {
            id: doc_id,
            path,
            changes,
            warnings,
            parent_relationship,
        });
    }

    updates.insert("updatedAt".to_string(), current_timestamp());
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let patched = patch_frontmatter_content(&content, &updates, &clear_fields)?;
    let patched = match accord_updates {
        Some(accord) => patch_accord_content(&patched, &accord)?,
        None => patched,
    };
    let patched = if let Some(body) = replacement_body {
        replace_markdown_body(&patched, body)?
    } else {
        patched
    };
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        workspace,
        "task.updated",
        &doc_id,
        &format!(
            "Updated {} metadata: {}",
            doc_id,
            changes
                .iter()
                .map(|change| change.field.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    )?;

    Ok(UpdateOutcome {
        id: doc_id,
        path,
        changes,
        warnings,
        parent_relationship,
    })
}

pub(crate) fn validate_update_options(
    options: &UpdateOptions,
    hierarchy: &HierarchyIndex,
) -> Result<Option<ParentRelationship>, Error> {
    if let Some(title) = options.title.as_deref() {
        require_nonempty(Some(title), "update --title must not be empty")?;
    }
    validate_task_kind_option(options.kind.as_deref(), "update --kind")?;
    validate_optional_vocabulary(
        options.priority.as_deref(),
        "update --priority",
        PRIORITIES,
        "priority",
    )?;
    validate_optional_vocabulary(
        options.effort.as_deref(),
        "update --effort",
        EFFORTS,
        "effort",
    )?;
    if options.assignee.is_some() {
        return Err(Error::usage(
            "update cannot write assignee; use accord claim or release",
        ));
    }
    if let Some(due_date) = options.due_date.as_deref() {
        require_nonempty(Some(due_date), "update --due-date must not be empty")?;
    }
    let parent_relationship = if let Some(parent) = options.parent.as_deref() {
        let parent = require_nonempty(Some(parent), "update --parent must not be empty")?;
        if parent == options.id {
            return Err(Error::user(format!(
                "Validation failed: task {} cannot be its own parent",
                options.id
            )));
        }
        hierarchy.validate_all_task_hierarchies()?;
        Some(resolve_parent_relationship(hierarchy, "task", parent)?)
    } else {
        None
    };
    for (field, values) in [
        ("--tag", &options.tags),
        ("--blocker", &options.blockers),
        ("--reference", &options.references),
        ("--related-file", &options.related_files),
    ] {
        for value in values {
            require_nonempty(Some(value), &format!("update {field} must not be empty"))?;
        }
    }
    for blocker in &options.blockers {
        if hierarchy.document(blocker).is_none() {
            return Err(Error::user(format!(
                "Validation failed: blocker document not found: {blocker}"
            )));
        }
    }
    Ok(parent_relationship)
}

fn validate_task_kind_option(kind: Option<&str>, flag: &str) -> Result<(), Error> {
    let Some(kind) = kind else {
        return Ok(());
    };
    let kind = require_nonempty(Some(kind), &format!("{flag} must not be empty"))?;
    validate_task_kind(kind).map_err(|message| Error::user(format!("Validation failed: {message}")))
}

fn validate_optional_vocabulary(
    value: Option<&str>,
    flag: &str,
    allowed: &[&str],
    label: &str,
) -> Result<(), Error> {
    let Some(value) = value else {
        return Ok(());
    };
    let value = require_nonempty(Some(value), &format!("{flag} must not be empty"))?;
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(Error::user(format!(
            "Validation failed: invalid {label} `{value}`; expected one of: {}",
            allowed.join(", ")
        )))
    }
}

fn apply_scalar_update(
    updates: &mut BTreeMap<String, String>,
    changes: &mut Vec<UpdateChange>,
    doc: &Document,
    key: &str,
    value: Option<&str>,
) -> Result<(), Error> {
    let Some(value) = value else {
        return Ok(());
    };
    let value = require_nonempty(Some(value), &format!("update --{key} must not be empty"))?;
    let old = doc.field(key).unwrap_or("");
    if old != value {
        updates.insert(key.to_string(), value.to_string());
        changes.push(UpdateChange {
            field: key.to_string(),
            old: old.to_string(),
            new: value.to_string(),
        });
    }
    Ok(())
}

fn apply_list_replace_update(
    updates: &mut BTreeMap<String, String>,
    changes: &mut Vec<UpdateChange>,
    doc: &Document,
    key: &str,
    additions: &[String],
) {
    if additions.is_empty() {
        return;
    }
    let old_values = doc.field(key).map(parse_field_values).unwrap_or_default();
    let new_values = additions.to_vec();
    if new_values != old_values {
        updates.insert(key.to_string(), inline_array(&new_values));
        changes.push(UpdateChange {
            field: key.to_string(),
            old: display_list_value(&old_values),
            new: display_list_value(&new_values),
        });
    }
}

/// Applies deterministic replacement (D36) to the accord definition fields.
///
/// Returns the accord record to write, or `None` when no definition field
/// changed. Accord lifecycle fields are read from the document and rewritten
/// unchanged, so an update never disturbs status or delivery data.
fn apply_accord_definition_update(
    changes: &mut Vec<UpdateChange>,
    doc: &Document,
    options: &UpdateOptions,
) -> Result<Option<AccordRecord>, Error> {
    let clears_acceptance = options
        .clear
        .iter()
        .any(|field| field == "acceptance" || field == "criterion");
    if clears_acceptance {
        return Err(Error::user(format!(
            "Validation failed: {} cannot clear acceptance; an active task requires at least one criterion",
            doc.id()
        )));
    }

    let mut accord = AccordRecord::from_document(doc, &current_timestamp());
    let mut changed = false;
    let mut apply = |changes: &mut Vec<UpdateChange>,
                     field: &str,
                     current: &mut Vec<String>,
                     replacement: &[String],
                     cleared: bool| {
        let new_values = if cleared {
            Vec::new()
        } else if replacement.is_empty() {
            return;
        } else {
            replacement.to_vec()
        };
        if new_values == *current {
            return;
        }
        changes.push(UpdateChange {
            field: field.to_string(),
            old: display_list_value(current),
            new: display_list_value(&new_values),
        });
        *current = new_values;
        changed = true;
    };

    let cleared = |name: &str| options.clear.iter().any(|field| field == name);
    let mut acceptance = accord.acceptance.clone();
    apply(
        changes,
        "acceptance",
        &mut acceptance,
        &options.acceptance,
        false,
    );
    let mut constraints = accord.constraints.clone();
    apply(
        changes,
        "constraints",
        &mut constraints,
        &options.constraints,
        cleared("constraint") || cleared("constraints"),
    );
    let mut validations = accord.validations.clone();
    apply(
        changes,
        "validation",
        &mut validations,
        &options.validations,
        cleared("validation") || cleared("validations"),
    );

    if !changed {
        return Ok(None);
    }
    accord.acceptance = acceptance;
    accord.constraints = constraints;
    accord.validations = validations;
    Ok(Some(accord))
}

fn display_list_value(values: &[String]) -> String {
    if values.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", values.join(", "))
    }
}

pub(crate) fn display_change_field(field: &str, relationship: Option<ParentRelationship>) -> &str {
    match field {
        "parentId" => relationship
            .unwrap_or(ParentRelationship::Parent)
            .human_label(),
        _ => field,
    }
}

pub(crate) fn display_change_value(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}

pub(crate) fn complete(
    workspace: &TandemProject,
    options: CompleteOptions,
) -> Result<CompleteOutcome, Error> {
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    let hierarchy = hierarchy_from_workspace(workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| Error::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(Error::user(format!(
            "Validation failed: only task documents can be completed in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;
    let active_descendants = active_task_descendant_ids(&hierarchy, doc.id());
    if !active_descendants.is_empty() {
        return Err(Error::user(format!(
            "Validation failed: cannot complete {} while it has active descendants: {}",
            doc.id(),
            active_descendants.join(", ")
        )));
    }
    let unresolved = unresolved_blockers_in_hierarchy(&hierarchy, doc.field("blockers"));
    if !unresolved.is_empty() {
        return Err(Error::user(format!(
            "Validation failed: {} has unresolved blockers: {}",
            doc.id(),
            unresolved.join(", ")
        )));
    }
    let completion_diagnostics = {
        let role = hierarchy
            .task_role(&doc)
            .map_err(|error| Error::user(error.message))?;
        let resolved_descendants = resolved_task_descendants(&hierarchy, doc.id());
        crate::protocol::diagnostic::completion_policy_diagnostics(
            &doc,
            role,
            &resolved_descendants,
        )
    };
    if let Some(error) = completion_diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == crate::protocol::diagnostic::Severity::Error)
    {
        return Err(Error::user(error.message.clone()));
    }
    let mut warnings = completion_diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == crate::protocol::diagnostic::Severity::Warning)
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
    let has_completion_warnings = !warnings.is_empty();
    warnings.extend(workspace_deprecation_warnings(workspace)?);
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    // Protocol 0.3.0 (D16/D39): completing a delivered Task atomically accepts
    // the Accord and archives it; there is no accepted-but-active state.
    let mut accord = AccordRecord::from_document(&doc, &now);
    if accord.status.eq_ignore_ascii_case("delivered") {
        accord.status = "accepted".to_string();
        accord.reviewer = options.reviewer.clone();
        accord.note = None;
        accord.reason = None;
    }
    let mut updates = BTreeMap::new();
    updates.insert("updatedAt".to_string(), now.clone());
    updates.insert("archivedAt".to_string(), now);
    let mut patched = patch_accord_content(&content, &accord)?;
    patched = patch_frontmatter_content(
        &patched,
        &updates,
        &[
            "state",
            "completedAt",
            "completionSummary",
            "completionValidation",
            "completionReviewer",
            "filesChanged",
        ],
    )?;
    let patched = patch_resolution_content(
        &patched,
        &ResolutionRecord {
            outcome: Some(RESOLUTION_OUTCOME_COMPLETED.to_string()),
            note: options.note,
            reviewer: options.reviewer,
        },
    )?;
    let summary = "Completed".to_string();
    let log_path = project::write::archive_board_document(
        workspace,
        &doc.path,
        &signature,
        &patched,
        "completed",
    )?;
    append_event(workspace, "task.completed", doc.id(), &summary)?;
    drop(_hierarchy_lock);
    let checkpoint = checkpoint_boundary();
    Ok(CompleteOutcome {
        id: doc.id().to_string(),
        board_path: doc.path,
        log_path,
        warnings,
        has_completion_warnings,
        checkpoint,
    })
}

pub(crate) fn cancel(
    workspace: &TandemProject,
    id: &str,
    reason: &str,
) -> Result<CancelOutcome, Error> {
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    let reason = require_nonempty(Some(reason), "cancel requires --reason <text>")?.to_string();
    let hierarchy = hierarchy_from_workspace(workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let doc = hierarchy
        .document(id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| Error::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(Error::user(format!(
            "Validation failed: only active task documents can be canceled: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;

    let active_descendants = active_task_descendant_ids(&hierarchy, doc.id());
    if !active_descendants.is_empty() {
        return Err(Error::user(format!(
            "Validation failed: cannot cancel {} while it has active descendants: {}",
            doc.id(),
            active_descendants.join(", ")
        )));
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut updates = BTreeMap::new();
    updates.insert("updatedAt".to_string(), now.clone());
    updates.insert("archivedAt".to_string(), now);
    let patched = patch_frontmatter_content(
        &content,
        &updates,
        &[
            "state",
            "completedAt",
            "completionSummary",
            "completionValidation",
            "completionReviewer",
            "filesChanged",
        ],
    )?;
    let patched = patch_resolution_content(
        &patched,
        &ResolutionRecord {
            outcome: Some(RESOLUTION_OUTCOME_CANCELED.to_string()),
            note: Some(reason.clone()),
            ..ResolutionRecord::default()
        },
    )?;
    let summary = format!("Canceled: {reason}");
    let log_path = project::write::archive_board_document(
        workspace, &doc.path, &signature, &patched, "canceled",
    )?;
    append_event(workspace, "task.canceled", doc.id(), &summary)?;
    drop(_hierarchy_lock);
    let checkpoint = checkpoint_boundary();

    Ok(CancelOutcome {
        id: doc.id().to_string(),
        reason,
        board_path: doc.path,
        log_path,
        checkpoint,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn references_resolve_tagged_tasks_while_parent_and_blockers_require_real_targets() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-task-papercut-reference-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.3.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        // A Papercut is a low-priority Task tagged papercut in protocol 0.3.0,
        // so its ID is a real document target for loose references.
        let tagged = add(
            &project,
            AddOptions {
                acceptance: vec!["friction captured".to_string()],
                title: Some("Small friction".to_string()),
                tags: vec!["papercut".to_string()],
                priority: Some("low".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let papercut_id = tagged.id;
        let update_papercut_id = add(
            &project,
            AddOptions {
                acceptance: vec!["friction captured".to_string()],
                title: Some("More friction".to_string()),
                tags: vec!["papercut".to_string()],
                priority: Some("low".to_string()),
                ..Default::default()
            },
        )
        .unwrap()
        .id;

        let created = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Fix the friction".to_string()),
                references: vec![papercut_id.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(created.warnings.is_empty());
        assert!(fs::read_to_string(&created.path)
            .unwrap()
            .contains(&format!("references: [\"{papercut_id}\"]")));

        let updated = update(
            &project,
            UpdateOptions {
                id: created.id.clone(),
                references: vec![update_papercut_id.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(updated.warnings.is_empty());
        let updated_source = fs::read_to_string(&updated.path).unwrap();
        assert!(!updated_source.contains(&papercut_id));
        assert!(updated_source.contains(&update_papercut_id));

        let parent_error = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Invalid parent".to_string()),
                parent: Some("task-99".to_string()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(parent_error.message.contains("parent document not found"));

        let blocker_error = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Invalid blocker".to_string()),
                blockers: vec!["task-99".to_string()],
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(blocker_error.message.contains("blocker document not found"));
        fs::remove_dir_all(project.root()).unwrap();
    }

    #[test]
    fn completion_rejects_active_task_descendants() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-complete-descendants-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.2.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        let parent = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Parent".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let child = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Child".to_string()),
                parent: Some(parent.id.clone()),
                ..Default::default()
            },
        )
        .unwrap();
        let error = complete(
            &project,
            CompleteOptions {
                id: parent.id.clone(),
                note: Some("Done".to_string()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(error.message.contains("cannot complete"));
        assert!(error.message.contains(&child.id));
        assert!(parent.path.exists());
        fs::remove_dir_all(project.root()).unwrap();
    }

    #[test]
    fn completion_rejects_epic_with_active_child_task() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-complete-epic-descendant-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.2.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        let epic = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Epic".to_string()),
                kind: Some("epic".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let child = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Epic task".to_string()),
                parent: Some(epic.id.clone()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(child.id, "task-2");
        let error = complete(
            &project,
            CompleteOptions {
                id: epic.id.clone(),
                note: Some("Done".to_string()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(error.message.contains("cannot complete"));
        assert!(error.message.contains(&child.id));
        assert!(epic.path.exists());
        fs::remove_dir_all(project.root()).unwrap();
    }

    #[test]
    fn completion_repairs_a_preexisting_orphan() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-complete-orphan-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.2.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        let parent = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Archived parent".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let child_path = project.tasks_dir.join("task-1-1.md");
        fs::write(
            &child_path,
            format!(
                "---\nid: task-1-1\ntype: task\ntitle: Orphan\nstate: todo\nparentId: {}\n---\n",
                parent.id
            ),
        )
        .unwrap();
        let archived_parent_path = project.logs_dir.join("task-1.md");
        fs::rename(&parent.path, &archived_parent_path).unwrap();

        let outcome = complete(
            &project,
            CompleteOptions {
                id: "task-1-1".to_string(),
                note: Some("Repaired orphan".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(outcome.log_path.exists());
        assert!(!child_path.exists());
        fs::remove_dir_all(project.root()).unwrap();
    }

    #[test]
    fn completion_preserves_unknown_source_and_returns_policy_warning() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-complete-warning-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.2.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        let created = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Keep source".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let source = fs::read_to_string(&created.path).unwrap();
        fs::write(
            &created.path,
            source.replacen("title:", "unknown: retain\ntitle:", 1),
        )
        .unwrap();
        let outcome = complete(
            &project,
            CompleteOptions {
                id: created.id.clone(),
                note: Some("Done".to_string()),
                ..CompleteOptions::default()
            },
        )
        .unwrap();
        assert!(outcome.has_completion_warnings);
        assert!(!outcome
            .warnings
            .iter()
            .any(|warning| warning.contains("validation.state")));
        let archived = fs::read_to_string(&outcome.log_path).unwrap();
        assert!(archived.contains("unknown: retain"));
        assert!(archived.contains("note: \"Done\""));
        assert!(archived.contains("outcome: \"completed\""));
        assert!(archived.contains("archivedAt"));
        assert!(!archived.contains("completion:"));
        fs::remove_dir_all(project.root()).unwrap();
    }

    #[test]
    fn update_writes_accord_definition_fields_and_reports_only_real_changes() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-update-accord-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.2.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        let created = add(
            &project,
            AddOptions {
                acceptance: vec!["original".to_string()],
                title: Some("Accord update".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let replaced = update(
            &project,
            UpdateOptions {
                id: created.id.clone(),
                acceptance: vec!["first".to_string(), "second".to_string()],
                constraints: vec!["no new deps".to_string()],
                validations: vec!["cargo test".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        let fields = replaced
            .changes
            .iter()
            .map(|change| change.field.as_str())
            .collect::<Vec<_>>();
        assert_eq!(fields, vec!["acceptance", "constraints", "validation"]);
        let content = std::fs::read_to_string(&replaced.path).unwrap();
        assert!(content.contains("first") && content.contains("second"));
        assert!(content.contains("no new deps"));
        assert!(content.contains("cargo test"));
        assert!(!content.contains("original"));

        let repeated = update(
            &project,
            UpdateOptions {
                id: created.id.clone(),
                acceptance: vec!["first".to_string(), "second".to_string()],
                constraints: vec!["no new deps".to_string()],
                validations: vec!["cargo test".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            repeated.changes.is_empty(),
            "an identical update must report no changes: {:?}",
            repeated.changes
        );

        let cleared = update(
            &project,
            UpdateOptions {
                id: created.id.clone(),
                clear: vec!["constraint".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            cleared
                .changes
                .iter()
                .map(|change| change.field.as_str())
                .collect::<Vec<_>>(),
            vec!["constraints"]
        );
        let content = std::fs::read_to_string(&cleared.path).unwrap();
        assert!(!content.contains("no new deps"));
        assert!(content.contains("first"));

        let rejected = update(
            &project,
            UpdateOptions {
                id: created.id.clone(),
                clear: vec!["acceptance".to_string()],
                ..Default::default()
            },
        );
        assert!(rejected.is_err(), "clearing acceptance must be rejected");

        std::fs::remove_dir_all(project.root()).unwrap();
    }

    #[test]
    fn add_rejects_archived_parent() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-add-archived-parent-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.2.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        let parent = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Parent".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        complete(
            &project,
            CompleteOptions {
                id: parent.id.clone(),
                note: Some("Archived".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let error = add(
            &project,
            AddOptions {
                acceptance: vec!["test acceptance".to_string()],
                title: Some("Archived child".to_string()),
                parent: Some(parent.id),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(error.message.contains("archived parent"));
        assert!(error.message.contains("Board"));
        fs::remove_dir_all(project.root()).unwrap();
    }
}
