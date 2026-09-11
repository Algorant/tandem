//! Exceptional human validation escalation.
use std::collections::BTreeMap;

use crate::app::support::{
    append_event, checkpoint_boundary, current_timestamp, hierarchy_from_project, validate_state,
    validate_task_document_against_hierarchy,
};
use crate::app::Error;
use crate::project::write::{ensure_file_unchanged, read_file_snapshot, HierarchyLock};
use crate::project::{patch_frontmatter_content, write_atomic, CheckpointOutcome, TandemProject};
use crate::protocol::accord;
use crate::protocol::hierarchy::{DocumentLocation, TaskRole};

#[derive(Debug, Default)]
pub(crate) struct ReviewOptions {
    pub(crate) id: String,
    pub(crate) criterion: Option<String>,
    pub(crate) reviewer: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) json: bool,
}

#[derive(Debug)]
pub(crate) struct ReviewOutcome {
    pub(crate) id: String,
    pub(crate) state: String,
    pub(crate) event_name: String,
    pub(crate) checkpoint: CheckpointOutcome,
}

pub(crate) fn transition(
    workspace: &TandemProject,
    action: &str,
    options: ReviewOptions,
) -> Result<ReviewOutcome, Error> {
    if action != "request" {
        return Err(Error::usage(
            "review accepts one escalation action; complete accepts validation",
        ));
    }
    let criterion = options
        .criterion
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| Error::usage("review requires --criterion <text>"))?
        .to_string();
    let note = options
        .note
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| Error::usage("review requires --note <text>"))?;
    let _lock = HierarchyLock::acquire(workspace)?;
    let hierarchy = hierarchy_from_project(workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| Error::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(Error::user(format!(
            "Review failed: {} is not a task",
            doc.id()
        )));
    }
    let role = hierarchy
        .task_role(&doc)?
        .ok_or_else(|| Error::user(format!("Review failed: {} has no task role", doc.id())))?;
    if role == TaskRole::Subtask {
        return Err(Error::user(format!(
            "Review failed: Subtasks cannot be reviewed: {}",
            doc.id()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;
    validate_state(workspace, "validation")?;
    accord::validate_review_criterion(doc.id(), &accord::acceptance(&doc), &criterion)
        .map_err(Error::user)?;
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut updates = BTreeMap::from([
        ("state".to_string(), "validation".to_string()),
        ("validation.criterion".to_string(), criterion.clone()),
        ("validation.note".to_string(), note.to_string()),
        ("validation.requestedAt".to_string(), now.clone()),
        ("updatedAt".to_string(), now),
    ]);
    if let Some(reviewer) = options.reviewer.filter(|value| !value.trim().is_empty()) {
        updates.insert("validation.reviewer".to_string(), reviewer);
    }
    let patched = patch_frontmatter_content(&content, &updates, &["review", "validation.state"])?;
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        workspace,
        "review.requested",
        doc.id(),
        &format!("Requested validation for {}", doc.id()),
    )?;
    drop(_lock);
    let checkpoint = checkpoint_boundary(workspace, &hierarchy, &doc);
    Ok(ReviewOutcome {
        id: doc.id().to_string(),
        state: "validation".to_string(),
        event_name: "review.requested".to_string(),
        checkpoint,
    })
}
