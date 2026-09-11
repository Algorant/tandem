//! Shared Decision creation and diagnostic orchestration.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::support::{
    append_event, create_new_sequential_document_in, current_timestamp, reference_target_exists,
};
use crate::app::tasks::UpdateChange;
use crate::app::Error;
use crate::project::write::{ensure_file_unchanged, read_file_snapshot, HierarchyLock};
use crate::project::{
    patch_frontmatter_content, replace_markdown_body, write_atomic, yaml_double_quote,
    StoredDocument, TandemProject,
};
use crate::protocol::document::{
    decision_status_sets_decided_at, is_absolute_reference_url, parse_field_values,
    validate_decision_status,
};
use crate::protocol::hierarchy::DocumentLocation;

#[derive(Debug, Default)]
pub(crate) struct AddOptions {
    pub(crate) title: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) deciders: Vec<String>,
    pub(crate) context: Option<String>,
    pub(crate) consequences: Vec<String>,
    pub(crate) alternatives: Vec<String>,
    pub(crate) supersedes: Vec<String>,
    pub(crate) superseded_by: Vec<String>,
    pub(crate) references: Vec<String>,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct AddOutcome {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) path: PathBuf,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct UpdateOptions {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) deciders: Vec<String>,
    pub(crate) supersedes: Vec<String>,
    pub(crate) references: Vec<String>,
    pub(crate) related_files: Vec<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) clear: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct UpdateOutcome {
    pub(crate) id: String,
    pub(crate) path: PathBuf,
    pub(crate) changes: Vec<UpdateChange>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct WithdrawOutcome {
    pub(crate) id: String,
    pub(crate) reason: String,
    pub(crate) path: PathBuf,
}

pub(crate) fn add(project: &TandemProject, options: AddOptions) -> Result<AddOutcome, Error> {
    let title = require_nonempty(
        options.title.as_deref(),
        "decision add requires --title <title>",
    )?
    .to_string();
    let status = options.status.as_deref().unwrap_or("proposed");
    validate_status(status)?;
    validate_options(&options)?;
    let warnings = diagnostics(project, &options)?;
    let now = current_timestamp();
    // Protocol 0.3.0 (D41): no manual decision date. decidedAt is written by
    // the lifecycle layer when a Decision reaches accepted/rejected; new
    // decisions start proposed without a date field.
    let created = create_new_sequential_document_in(
        project,
        &project.decisions_dir(),
        "decision",
        |decision_id| {
            let mut lines = vec![
                "---".to_string(),
                format!("id: {decision_id}"),
                "type: decision".to_string(),
                format!("title: {}", yaml_double_quote(&title)),
                format!("status: {}", yaml_double_quote(status)),
            ];
            push_array_line(&mut lines, "deciders", &options.deciders);
            push_optional_line(&mut lines, "context", options.context.as_deref());
            push_array_line(&mut lines, "consequences", &options.consequences);
            push_array_line(&mut lines, "alternatives", &options.alternatives);
            push_array_line(&mut lines, "supersedes", &options.supersedes);
            push_array_line(&mut lines, "supersededBy", &options.superseded_by);
            push_array_line(&mut lines, "references", &options.references);
            push_array_line(&mut lines, "tags", &options.tags);
            lines.push(format!("createdAt: {}", yaml_double_quote(&now)));
            lines.push(format!("updatedAt: {}", yaml_double_quote(&now)));
            // D41: a Decision created directly in a terminal status carries the
            // same automatic decision date an `update` transition would write.
            if decision_status_sets_decided_at(status) {
                lines.push(format!("decidedAt: {}", yaml_double_quote(&now)));
            }
            lines.push("---".to_string());
            lines.push(String::new());
            if let Some(body) = options.body.as_deref() {
                lines.push(body.to_string());
            }
            lines.push(String::new());
            lines.join("\n")
        },
    )?;
    append_event(project, "decision.created", &created.id, &title)?;
    Ok(AddOutcome {
        id: created.id,
        title,
        status: status.to_string(),
        path: created.path,
        warnings,
    })
}

pub(crate) fn update(
    project: &TandemProject,
    options: UpdateOptions,
) -> Result<UpdateOutcome, Error> {
    // Resolve and mutate active records under the existing write safety lock.
    let _lock = HierarchyLock::acquire(project)?;
    let doc = active_decision(project, &options.id)?;
    let clear_fields = resolve_clear_fields(&options.clear)?;
    validate_update_options(&options, &clear_fields)?;

    // One captured timestamp is shared by `decidedAt` and `updatedAt` so a
    // terminal transition records a single coherent instant.
    let now = current_timestamp();
    let mut updates = BTreeMap::new();
    let mut removes: Vec<&str> = Vec::new();
    let mut changes = Vec::new();

    apply_decision_scalar(
        &mut updates,
        &mut changes,
        &doc,
        "title",
        options.title.as_deref(),
    )?;
    apply_decision_status(
        &mut updates,
        &mut changes,
        &doc,
        options.status.as_deref(),
        &now,
    );
    apply_decision_list(
        &mut updates,
        &mut changes,
        &doc,
        "deciders",
        &options.deciders,
    );
    apply_decision_list(
        &mut updates,
        &mut changes,
        &doc,
        "supersedes",
        &options.supersedes,
    );
    apply_decision_list(
        &mut updates,
        &mut changes,
        &doc,
        "references",
        &options.references,
    );
    apply_decision_list(
        &mut updates,
        &mut changes,
        &doc,
        "relatedFiles",
        &options.related_files,
    );
    apply_decision_list(&mut updates, &mut changes, &doc, "tags", &options.tags);

    let clear_body = clear_fields.contains(&"body");
    for field in &clear_fields {
        if *field == "body" {
            continue;
        }
        if doc.field(field).is_some() {
            removes.push(field);
            changes.push(UpdateChange {
                field: (*field).to_string(),
                old: "<set>".to_string(),
                new: "<cleared>".to_string(),
            });
        }
    }

    let replacement_body = if clear_body {
        Some("")
    } else {
        options.body.as_deref()
    };
    let apply_body = replacement_body.filter(|body| doc.body.as_str() != *body);
    if apply_body.is_some() {
        changes.push(UpdateChange {
            field: "body".to_string(),
            old: "<body>".to_string(),
            new: "<body>".to_string(),
        });
    }

    let warnings = diagnostics_for(project, &options.references, &options.supersedes, &[])?;
    let doc_id = doc.id().to_string();
    let path = doc.path.clone();
    if changes.is_empty() {
        return Ok(UpdateOutcome {
            id: doc_id,
            path,
            changes,
            warnings,
        });
    }

    updates.insert("updatedAt".to_string(), now);
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let patched = patch_frontmatter_content(&content, &updates, &removes)?;
    let patched = if let Some(body) = apply_body {
        replace_markdown_body(&patched, body)?
    } else {
        patched
    };
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        project,
        "decision.updated",
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
    })
}

/// Resolves the supported `--clear` aliases to canonical document fields.
///
/// The accepted names follow the existing `update` clear convention: canonical
/// field names plus the established `tag`/`related-file` aliases. Unknown,
/// immutable (`title`, `id`, `type`, timestamps), and lifecycle (`status`)
/// clears are rejected before any read or write.
fn resolve_clear_fields(clear: &[String]) -> Result<Vec<&'static str>, Error> {
    let mut fields = Vec::new();
    for requested in clear {
        let field = match requested.as_str() {
            "body" => "body",
            "tags" | "tag" => "tags",
            "references" => "references",
            "relatedFiles" | "related-file" => "relatedFiles",
            "deciders" => "deciders",
            "supersedes" => "supersedes",
            _ => {
                return Err(Error::usage(format!(
                    "update --clear does not support field `{requested}` for decision documents"
                )));
            }
        };
        if !fields.contains(&field) {
            fields.push(field);
        }
    }
    Ok(fields)
}

fn validate_update_options(
    options: &UpdateOptions,
    clear_fields: &[&'static str],
) -> Result<(), Error> {
    if let Some(title) = options.title.as_deref() {
        require_nonempty(Some(title), "update --title must not be empty")?;
    }
    if let Some(status) = options.status.as_deref() {
        validate_status(status)?;
    }
    if let Some(body) = options.body.as_deref() {
        require_nonempty(
            Some(body),
            "update --body must not be empty; use --clear body to remove the body",
        )?;
    }
    for (flag, values) in [
        ("--decider", &options.deciders),
        ("--supersedes", &options.supersedes),
        ("--reference", &options.references),
        ("--related-file", &options.related_files),
        ("--tag", &options.tags),
    ] {
        for value in values {
            require_nonempty(Some(value), &format!("update {flag} must not be empty"))?;
        }
    }
    // A field may be replaced or cleared, never both in one request.
    for (field, supplied, flag) in [
        ("body", options.body.is_some(), "--body"),
        ("tags", !options.tags.is_empty(), "--tag"),
        ("references", !options.references.is_empty(), "--reference"),
        (
            "relatedFiles",
            !options.related_files.is_empty(),
            "--related-file",
        ),
        ("deciders", !options.deciders.is_empty(), "--decider"),
        ("supersedes", !options.supersedes.is_empty(), "--supersedes"),
    ] {
        if supplied && clear_fields.contains(&field) {
            return Err(Error::usage(format!(
                "update cannot combine {flag} with --clear {field}"
            )));
        }
    }
    Ok(())
}

fn apply_decision_scalar(
    updates: &mut BTreeMap<String, String>,
    changes: &mut Vec<UpdateChange>,
    doc: &StoredDocument,
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

/// Applies a Decision status change.
///
/// `decidedAt` records the latest actual transition into `accepted` or
/// `rejected`; it is retained when the status later moves to `proposed`,
/// `deprecated`, or `superseded`. Repeating an unchanged status is a true
/// no-op and never backfills a historical record. The caller supplies the one
/// captured update timestamp so `decidedAt` and `updatedAt` agree.
fn apply_decision_status(
    updates: &mut BTreeMap<String, String>,
    changes: &mut Vec<UpdateChange>,
    doc: &StoredDocument,
    status: Option<&str>,
    now: &str,
) {
    let Some(status) = status else {
        return;
    };
    let old = doc.field("status").unwrap_or("");
    if old == status {
        return;
    }
    updates.insert("status".to_string(), status.to_string());
    changes.push(UpdateChange {
        field: "status".to_string(),
        old: old.to_string(),
        new: status.to_string(),
    });
    if decision_status_sets_decided_at(status) {
        updates.insert("decidedAt".to_string(), now.to_string());
        changes.push(UpdateChange {
            field: "decidedAt".to_string(),
            old: doc.field("decidedAt").unwrap_or("").to_string(),
            new: now.to_string(),
        });
    }
}

fn apply_decision_list(
    updates: &mut BTreeMap<String, String>,
    changes: &mut Vec<UpdateChange>,
    doc: &StoredDocument,
    key: &str,
    values: &[String],
) {
    if values.is_empty() {
        return;
    }
    let old = doc.field(key).map(parse_field_values).unwrap_or_default();
    if values != old.as_slice() {
        updates.insert(key.to_string(), inline_array(values));
        changes.push(UpdateChange {
            field: key.to_string(),
            old: display_list(&old),
            new: display_list(values),
        });
    }
}

fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", values.join(", "))
    }
}

pub(crate) fn withdraw(
    project: &TandemProject,
    id: &str,
    reason: String,
) -> Result<WithdrawOutcome, Error> {
    let doc = active_decision(project, id)?;
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let updates = BTreeMap::from([
        ("status".to_string(), "withdrawn".to_string()),
        ("withdrawnAt".to_string(), now.clone()),
        ("withdrawalReason".to_string(), reason.clone()),
        ("updatedAt".to_string(), now),
    ]);
    let patched = patch_frontmatter_content(&content, &updates, &[])?;
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        project,
        "decision.withdrawn",
        doc.id(),
        &format!("Withdrew decision {}: {reason}", doc.id()),
    )?;
    Ok(WithdrawOutcome {
        id: doc.id().to_string(),
        reason,
        path: doc.path,
    })
}

fn active_decision(project: &TandemProject, id: &str) -> Result<StoredDocument, Error> {
    match project.find_document(id)? {
        Some(doc) if doc.doc_type() != "decision" => Err(Error::user(format!(
            "Validation failed: only decision documents can be updated here: {} is type {}",
            doc.id(),
            doc.doc_type()
        ))),
        Some(doc) if doc.location != DocumentLocation::Board => Err(Error::user(format!(
            "Validation failed: decision {id} is archived and cannot be updated"
        ))),
        Some(doc) => Ok(doc),
        None => Err(Error::user(format!("active decision not found: {id}"))),
    }
}

fn validate_options(options: &AddOptions) -> Result<(), Error> {
    if let Some(context) = options.context.as_deref() {
        require_nonempty(Some(context), "decision add --context must not be empty")?;
    }
    for (flag, values) in [
        ("--decider", &options.deciders),
        ("--consequence", &options.consequences),
        ("--alternative", &options.alternatives),
        ("--supersedes", &options.supersedes),
        ("--superseded-by", &options.superseded_by),
        ("--reference", &options.references),
        ("--tag", &options.tags),
    ] {
        for value in values {
            require_nonempty(
                Some(value),
                &format!("decision add {flag} must not be empty"),
            )?;
        }
    }
    Ok(())
}

pub(crate) fn validate_status(status: &str) -> Result<(), Error> {
    if status.trim().is_empty() {
        return Err(Error::usage("decision status must not be empty"));
    }
    validate_decision_status(status)
        .map_err(|message| Error::user(format!("Validation failed: {message}")))
}

pub(crate) fn diagnostics(
    project: &TandemProject,
    options: &AddOptions,
) -> Result<Vec<String>, Error> {
    diagnostics_for(
        project,
        &options.references,
        &options.supersedes,
        &options.superseded_by,
    )
}

/// Shared reference/supersession diagnostics for Decision add and update.
fn diagnostics_for(
    project: &TandemProject,
    references: &[String],
    supersedes: &[String],
    superseded_by: &[String],
) -> Result<Vec<String>, Error> {
    let mut warnings = Vec::new();
    for reference in references {
        if is_absolute_reference_url(reference) {
            continue;
        }
        if !reference_target_exists(project, reference)? {
            warnings.push(format!("reference not found: {reference}"));
        }
    }
    for target in supersedes {
        push_reference_warning(project, &mut warnings, "supersedes", target)?;
    }
    for target in superseded_by {
        push_reference_warning(project, &mut warnings, "supersededBy", target)?;
    }
    Ok(warnings)
}

fn push_reference_warning(
    project: &TandemProject,
    warnings: &mut Vec<String>,
    field: &str,
    id: &str,
) -> Result<(), Error> {
    match project.find_document(id)? {
        Some(doc) if doc.doc_type() == "decision" => {}
        Some(doc) => warnings.push(format!(
            "{field} target {id} is type {}, not decision",
            doc.doc_type()
        )),
        None => warnings.push(format!("{field} decision not found: {id}")),
    }
    Ok(())
}

fn require_nonempty<'a>(value: Option<&'a str>, message: &str) -> Result<&'a str, Error> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Error::usage(message))?;
    Ok(value)
}
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn creation_preserves_adr_input_and_returns_reference_diagnostics() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-decision-{}",
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
        let papercut_id = crate::app::tasks::add(
            &project,
            crate::app::tasks::AddOptions {
                acceptance: vec!["friction captured".to_string()],
                title: Some("Decision friction".to_string()),
                tags: vec!["papercut".to_string()],
                priority: Some("low".to_string()),
                ..Default::default()
            },
        )
        .unwrap()
        .id;
        let outcome = add(
            &project,
            AddOptions {
                title: Some("Choose seam".to_string()),
                body: Some("## Decision\nKeep bytes.  ".to_string()),
                deciders: vec!["A".to_string()],
                references: vec![
                    papercut_id.clone(),
                    "missing-task".to_string(),
                    "https://example.com/decisions/9".to_string(),
                    "HTTPS://Example.COM/Upper?x=1#fragment".to_string(),
                ],
                supersedes: vec!["missing-decision".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(outcome.id, "decision-1");
        assert_eq!(
            outcome.warnings,
            vec![
                "reference not found: missing-task".to_string(),
                "supersedes decision not found: missing-decision".to_string()
            ]
        );
        let source = fs::read_to_string(outcome.path).unwrap();
        assert!(source.contains("deciders: [\"A\"]"));
        assert!(source.contains(&format!(
            "references: [\"{papercut_id}\", \"missing-task\", \"https://example.com/decisions/9\", \"HTTPS://Example.COM/Upper?x=1#fragment\"]"
        )));
        assert!(source.contains("## Decision\nKeep bytes.  \n"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn creation_in_terminal_status_sets_decided_at() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-decision-dates-{}",
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
        let accepted = add(
            &project,
            AddOptions {
                title: Some("Accepted on creation".to_string()),
                status: Some("accepted".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(fs::read_to_string(&accepted.path)
            .unwrap()
            .contains("decidedAt:"));
        let proposed = add(
            &project,
            AddOptions {
                title: Some("Still proposed".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!fs::read_to_string(&proposed.path)
            .unwrap()
            .contains("decidedAt:"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn update_retains_historical_decided_at_and_repeat_status_is_a_noop() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-decision-retain-{}",
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
        let created = add(
            &project,
            AddOptions {
                title: Some("Dates".to_string()),
                status: Some("accepted".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let pinned = fs::read_to_string(&created.path)
            .unwrap()
            .lines()
            .map(|line| {
                if line.starts_with("decidedAt:") {
                    "decidedAt: \"2000-01-01T00:00:00Z\"".to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        fs::write(&created.path, &pinned).unwrap();

        // Repeating the unchanged status must not backfill or rewrite.
        let repeated = update(
            &project,
            UpdateOptions {
                id: created.id.clone(),
                status: Some("accepted".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(repeated.changes.is_empty());
        assert_eq!(fs::read_to_string(&repeated.path).unwrap(), pinned);

        // Leaving the terminal status retains the historical decision date.
        let deprecated = update(
            &project,
            UpdateOptions {
                id: created.id.clone(),
                status: Some("deprecated".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            deprecated
                .changes
                .iter()
                .map(|change| change.field.as_str())
                .collect::<Vec<_>>(),
            vec!["status"]
        );
        assert!(fs::read_to_string(&deprecated.path)
            .unwrap()
            .contains("decidedAt: \"2000-01-01T00:00:00Z\""));

        // Re-entering a terminal status refreshes the automatic date.
        let rejected = update(
            &project,
            UpdateOptions {
                id: created.id.clone(),
                status: Some("rejected".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            rejected
                .changes
                .iter()
                .map(|change| change.field.as_str())
                .collect::<Vec<_>>(),
            vec!["status", "decidedAt"]
        );
        assert!(!fs::read_to_string(&rejected.path)
            .unwrap()
            .contains("2000-01-01T00:00:00Z"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn update_rejects_wrong_type_and_missing_targets() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-decision-wrong-type-{}",
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
        crate::app::tasks::add(
            &project,
            crate::app::tasks::AddOptions {
                acceptance: vec!["ok".to_string()],
                title: Some("Task".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let wrong_type = update(
            &project,
            UpdateOptions {
                id: "task-1".to_string(),
                title: Some("Renamed".to_string()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(wrong_type.message.contains("only decision documents"));
        let missing = update(
            &project,
            UpdateOptions {
                id: "decision-9".to_string(),
                title: Some("Renamed".to_string()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(missing.message.contains("active decision not found"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn update_rejects_padded_status_and_shares_terminal_timestamp() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-decision-padded-{}",
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
        let created = add(
            &project,
            AddOptions {
                title: Some("Padded".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let before = fs::read_to_string(&created.path).unwrap();
        for padded in [" accepted ", "\taccepted", "accepted\n", " rejected"] {
            let error = update(
                &project,
                UpdateOptions {
                    id: created.id.clone(),
                    status: Some(padded.to_string()),
                    ..Default::default()
                },
            )
            .unwrap_err();
            assert!(
                error.message.contains("whitespace"),
                "{padded:?}: {}",
                error.message
            );
            assert_eq!(fs::read_to_string(&created.path).unwrap(), before);
        }
        let add_error = add(
            &project,
            AddOptions {
                title: Some("Bad creation".to_string()),
                status: Some(" accepted ".to_string()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(add_error.message.contains("whitespace"));

        let accepted = update(
            &project,
            UpdateOptions {
                id: created.id.clone(),
                status: Some("accepted".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let source = fs::read_to_string(&accepted.path).unwrap();
        let value = |prefix: &str| {
            source
                .lines()
                .find(|line| line.starts_with(prefix))
                .map(|line| line[prefix.len()..].trim().trim_matches('"').to_string())
                .unwrap_or_default()
        };
        assert!(!value("decidedAt:").is_empty());
        assert_eq!(value("decidedAt:"), value("updatedAt:"));
        fs::remove_dir_all(root).unwrap();
    }
}
