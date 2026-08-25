//! Shared review request and resolution operations.

use std::collections::BTreeMap;

use crate::app::support::{
    append_event, current_timestamp, hierarchy_from_project, validate_state,
    validate_task_document_against_hierarchy,
};
use crate::project::write::{ensure_file_unchanged, read_file_snapshot, HierarchyLock};
use crate::project::{
    patch_frontmatter_content, write_atomic, StoredDocument as Document, TandemProject,
};
use crate::protocol::hierarchy::TaskRole;
use crate::protocol::review;
use crate::CliError;

#[derive(Debug, Default)]
pub(crate) struct ReviewOptions {
    pub(crate) id: String,
    pub(crate) reviewer: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) json: bool,
}

#[derive(Debug)]
pub(crate) struct ReviewOutcome {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) state: String,
    pub(crate) event_name: String,
}

pub(crate) fn transition(
    workspace: &TandemProject,
    action: &str,
    options: ReviewOptions,
) -> Result<ReviewOutcome, CliError> {
    let _lock = HierarchyLock::acquire(workspace)?;
    let hierarchy = hierarchy_from_project(workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == crate::protocol::hierarchy::DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Review failed: {} is not a task",
            doc.id()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;
    let role = hierarchy
        .task_role(&doc)?
        .ok_or_else(|| CliError::user(format!("Review failed: {} has no task role", doc.id())))?;
    if action == "request" && role == TaskRole::Subtask {
        return Err(CliError::user(format!(
            "Review failed: Subtasks cannot be reviewed: {}",
            doc.id()
        )));
    }
    if action != "request" {
        if review::status(&doc) != Some("pending") {
            return Err(CliError::user(format!(
                "Review failed: {} does not have review.status: pending",
                doc.id()
            )));
        }
        return resolve(
            workspace,
            &doc,
            action,
            options.reviewer.as_deref(),
            options.note.as_deref(),
        );
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    validate_state(workspace, "validation")?;
    let mut updates = BTreeMap::new();
    updates.insert("review.status".to_string(), "pending".to_string());
    updates.insert("review.requestedAt".to_string(), now.clone());
    updates.insert("state".to_string(), "validation".to_string());
    updates.insert("updatedAt".to_string(), now);
    if let Some(reviewer) = options.reviewer.filter(|v| !v.trim().is_empty()) {
        updates.insert("review.reviewer".to_string(), reviewer);
    }
    if let Some(note) = options.note.filter(|v| !v.trim().is_empty()) {
        updates.insert("review.note".to_string(), note);
    }
    let patched = patch_frontmatter_content(&content, &updates, &[])?;
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        workspace,
        "review.requested",
        doc.id(),
        &format!("Requested review for {}", doc.id()),
    )?;
    Ok(ReviewOutcome {
        id: doc.id().to_string(),
        status: "pending".to_string(),
        state: "validation".to_string(),
        event_name: "review.requested".to_string(),
    })
}

fn resolve(
    workspace: &TandemProject,
    doc: &Document,
    action: &str,
    reviewer: Option<&str>,
    note: Option<&str>,
) -> Result<ReviewOutcome, CliError> {
    let (status, event_name, state) = match action {
        "accept" => ("accepted", "review.accepted", "validation"),
        "changes" => (
            "changes-requested",
            "review.changes_requested",
            "in-progress",
        ),
        "reject" => ("rejected", "review.rejected", "in-progress"),
        _ => return Err(CliError::usage(format!("unknown review action `{action}`"))),
    };
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut updates = BTreeMap::new();
    updates.insert("review.status".to_string(), status.to_string());
    if let Some(reviewer) = reviewer.filter(|value| !value.trim().is_empty()) {
        updates.insert("review.reviewer".to_string(), reviewer.to_string());
    }
    updates.insert("review.decidedAt".to_string(), now.clone());
    updates.insert("state".to_string(), state.to_string());
    updates.insert("updatedAt".to_string(), now);
    if let Some(note) = note.filter(|v| !v.trim().is_empty()) {
        updates.insert("review.note".to_string(), note.to_string());
    }
    let patched = patch_frontmatter_content(&content, &updates, &[])?;
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        workspace,
        event_name,
        doc.id(),
        &format!("Resolved review for {} as {}", doc.id(), status),
    )?;
    Ok(ReviewOutcome {
        id: doc.id().to_string(),
        status: status.to_string(),
        state: state.to_string(),
        event_name: event_name.to_string(),
    })
}
