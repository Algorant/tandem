//! Shared Task lifecycle operations.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::support::{
    active_task_descendant_ids, append_event, current_timestamp,
    hierarchy_from_project as hierarchy_from_workspace, require_nonempty,
    resolve_parent_relationship, unresolved_blockers_in_hierarchy, validate_state,
    validate_task_document_against_hierarchy, workspace_deprecation_warnings,
};
use crate::project::write::{ensure_file_unchanged, read_file_snapshot};
use crate::project::{
    self, patch_accord_content, patch_completion_content, patch_frontmatter_content,
    replace_markdown_body, write_atomic, yaml_double_quote, ProjectHierarchy as HierarchyIndex,
    StoredDocument as Document, TandemProject,
};
use crate::protocol::accord::{status as accord_status, AccordRecord};
use crate::protocol::document::{parse_field_values, validate_task_kind, EFFORTS, PRIORITIES};
use crate::protocol::hierarchy::{DocumentLocation, ParentRelationship};
use crate::protocol::ids::next_sequential_number as next_sequential_number_for_ids;
use crate::protocol::workflow::{CompletionRecord, COMPLETION_OUTCOME_CANCELED};
use crate::CliError;

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

#[derive(Debug, Default)]
pub(crate) struct AddOptions {
    pub(crate) title: Option<String>,
    pub(crate) state: Option<String>,
    pub(crate) json: bool,
    pub(crate) description: Option<String>,
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
    pub(crate) summary: Option<String>,
    pub(crate) files_changed: Vec<String>,
    pub(crate) validation: Option<String>,
    pub(crate) reviewer: Option<String>,
}

#[derive(Debug)]
pub(crate) struct CompleteOutcome {
    pub(crate) id: String,
    pub(crate) board_path: PathBuf,
    pub(crate) log_path: PathBuf,
    pub(crate) warnings: Vec<String>,
    pub(crate) has_completion_warnings: bool,
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
}

/// Create a Task, Epic, or Subtask after canonical hierarchy validation.
pub(crate) fn add(workspace: &TandemProject, options: AddOptions) -> Result<AddOutcome, CliError> {
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    let title =
        require_nonempty(options.title.as_deref(), "add requires --title <title>")?.to_string();
    let state = options.state.as_deref().unwrap_or("todo").to_string();
    validate_state(workspace, &state)?;
    validate_task_kind_option(options.kind.as_deref(), "add --kind")?;
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
        return Err(CliError::user(
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
    for blocker in &options.blockers {
        if hierarchy.document(blocker).is_none() {
            return Err(CliError::user(format!(
                "Validation failed: blocker document not found: {blocker}"
            )));
        }
    }

    let mut warnings = Vec::new();
    for reference in &options.references {
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
) -> Result<MoveTaskOutcome, CliError> {
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    validate_state(workspace, state)?;

    let hierarchy = hierarchy_from_workspace(workspace)?;
    let doc = hierarchy
        .document(id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
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
) -> Result<UpdateOutcome, CliError> {
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    let hierarchy = hierarchy_from_workspace(workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
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
        return Err(CliError::user(format!(
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
        "assignee",
        options.assignee.as_deref(),
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
    apply_list_append_update(&mut updates, &mut changes, &doc, "tags", &options.tags);
    apply_list_append_update(
        &mut updates,
        &mut changes,
        &doc,
        "blockers",
        &options.blockers,
    );
    apply_list_append_update(
        &mut updates,
        &mut changes,
        &doc,
        "references",
        &options.references,
    );
    apply_list_append_update(
        &mut updates,
        &mut changes,
        &doc,
        "relatedFiles",
        &options.related_files,
    );
    let replacement_body = options
        .body
        .as_deref()
        .filter(|body| doc.body.as_str() != *body);
    if replacement_body.is_some() {
        changes.push(UpdateChange {
            field: "body".to_string(),
            old: "<body>".to_string(),
            new: "<body>".to_string(),
        });
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
    let patched = patch_frontmatter_content(&content, &updates, &[])?;
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
) -> Result<Option<ParentRelationship>, CliError> {
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
    if let Some(assignee) = options.assignee.as_deref() {
        require_nonempty(Some(assignee), "update --assignee must not be empty")?;
    }
    if let Some(due_date) = options.due_date.as_deref() {
        require_nonempty(Some(due_date), "update --due-date must not be empty")?;
    }
    let parent_relationship = if let Some(parent) = options.parent.as_deref() {
        let parent = require_nonempty(Some(parent), "update --parent must not be empty")?;
        if parent == options.id {
            return Err(CliError::user(format!(
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
            return Err(CliError::user(format!(
                "Validation failed: blocker document not found: {blocker}"
            )));
        }
    }
    Ok(parent_relationship)
}

fn validate_task_kind_option(kind: Option<&str>, flag: &str) -> Result<(), CliError> {
    let Some(kind) = kind else {
        return Ok(());
    };
    let kind = require_nonempty(Some(kind), &format!("{flag} must not be empty"))?;
    validate_task_kind(kind)
        .map_err(|message| CliError::user(format!("Validation failed: {message}")))
}

fn validate_optional_vocabulary(
    value: Option<&str>,
    flag: &str,
    allowed: &[&str],
    label: &str,
) -> Result<(), CliError> {
    let Some(value) = value else {
        return Ok(());
    };
    let value = require_nonempty(Some(value), &format!("{flag} must not be empty"))?;
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(CliError::user(format!(
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
) -> Result<(), CliError> {
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

fn apply_list_append_update(
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
    let mut new_values = old_values.clone();
    for addition in additions {
        if !new_values.iter().any(|value| value == addition) {
            new_values.push(addition.to_string());
        }
    }
    if new_values != old_values {
        updates.insert(key.to_string(), inline_array(&new_values));
        changes.push(UpdateChange {
            field: key.to_string(),
            old: display_list_value(&old_values),
            new: display_list_value(&new_values),
        });
    }
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
) -> Result<CompleteOutcome, CliError> {
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    let summary = require_nonempty(
        options.summary.as_deref(),
        "complete requires --summary <text>",
    )?
    .to_string();
    let hierarchy = hierarchy_from_workspace(workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can be completed in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;
    let unresolved = unresolved_blockers_in_hierarchy(&hierarchy, doc.field("blockers"));
    if !unresolved.is_empty() {
        return Err(CliError::user(format!(
            "Validation failed: {} has unresolved blockers: {}",
            doc.id(),
            unresolved.join(", ")
        )));
    }
    let mut warnings = crate::protocol::diagnostic::completion_policy_diagnostics(&doc)
        .into_iter()
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
    let has_completion_warnings = !warnings.is_empty();
    warnings.extend(workspace_deprecation_warnings(workspace)?);
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut updates = BTreeMap::new();
    updates.insert("completedAt".to_string(), now.clone());
    updates.insert("updatedAt".to_string(), now);
    let patched = patch_frontmatter_content(
        &content,
        &updates,
        &[
            "state",
            "completionSummary",
            "completionValidation",
            "completionReviewer",
            "filesChanged",
        ],
    )?;
    let patched = patch_completion_content(
        &patched,
        &CompletionRecord {
            summary: summary.clone(),
            files_changed: options.files_changed,
            validation: options.validation,
            reviewer: options.reviewer,
            ..CompletionRecord::default()
        },
    )?;
    let log_path = project::write::archive_board_document(
        workspace,
        &doc.path,
        &signature,
        &patched,
        "completed",
    )?;
    append_event(workspace, "task.completed", doc.id(), &summary)?;
    Ok(CompleteOutcome {
        id: doc.id().to_string(),
        board_path: doc.path,
        log_path,
        warnings,
        has_completion_warnings,
    })
}

pub(crate) fn cancel(
    workspace: &TandemProject,
    id: &str,
    reason: &str,
) -> Result<CancelOutcome, CliError> {
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    let reason = require_nonempty(Some(reason), "cancel requires --reason <text>")?.to_string();
    let hierarchy = hierarchy_from_workspace(workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let doc = hierarchy
        .document(id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only active task documents can be canceled: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;

    let active_descendants = active_task_descendant_ids(&hierarchy, doc.id());
    if !active_descendants.is_empty() {
        return Err(CliError::user(format!(
            "Validation failed: cannot cancel {} while it has active descendants: {}",
            doc.id(),
            active_descendants.join(", ")
        )));
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut updates = BTreeMap::new();
    updates.insert("completedAt".to_string(), now.clone());
    updates.insert("updatedAt".to_string(), now);
    let patched = patch_frontmatter_content(
        &content,
        &updates,
        &[
            "state",
            "completionSummary",
            "completionValidation",
            "completionReviewer",
            "filesChanged",
        ],
    )?;
    let summary = format!("Canceled: {reason}");
    let patched = patch_completion_content(
        &patched,
        &CompletionRecord {
            summary: summary.clone(),
            outcome: Some(COMPLETION_OUTCOME_CANCELED.to_string()),
            ..CompletionRecord::default()
        },
    )?;
    let log_path = project::write::archive_board_document(
        workspace, &doc.path, &signature, &patched, "canceled",
    )?;
    append_event(workspace, "task.canceled", doc.id(), &summary)?;

    Ok(CancelOutcome {
        id: doc.id().to_string(),
        reason,
        board_path: doc.path,
        log_path,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn task_references_accept_papercuts_while_parent_and_blockers_remain_documents() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-task-papercut-reference-{}",
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
        let papercut = crate::app::papercuts::add(
            &project,
            crate::app::papercuts::AddOptions {
                title: Some("Small friction".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let papercut_id = papercut.papercut.id().to_string();
        let update_papercut_id = crate::app::papercuts::add(
            &project,
            crate::app::papercuts::AddOptions {
                title: Some("More friction".to_string()),
                ..Default::default()
            },
        )
        .unwrap()
        .papercut
        .id()
        .to_string();

        let created = add(
            &project,
            AddOptions {
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
        assert!(updated_source.contains(&papercut_id));
        assert!(updated_source.contains(&update_papercut_id));

        let parent_error = add(
            &project,
            AddOptions {
                title: Some("Invalid parent".to_string()),
                parent: Some(papercut_id.clone()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(parent_error.message.contains("parent document not found"));

        let blocker_error = add(
            &project,
            AddOptions {
                title: Some("Invalid blocker".to_string()),
                blockers: vec![papercut_id],
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(blocker_error.message.contains("blocker document not found"));
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
                id: created.id,
                summary: Some("Done".to_string()),
                ..CompleteOptions::default()
            },
        )
        .unwrap();
        assert!(outcome.has_completion_warnings);
        assert!(outcome
            .warnings
            .iter()
            .any(|warning| warning.contains("review.status")));
        let archived = fs::read_to_string(&outcome.log_path).unwrap();
        assert!(archived.contains("unknown: retain"));
        assert!(archived.contains("summary: \"Done\""));
        fs::remove_dir_all(project.root()).unwrap();
    }
}
