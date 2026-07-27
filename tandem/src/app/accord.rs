//! Shared accord and Validation lifecycle operations.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::project::{self, write_atomic};
use crate::protocol::accord::{self, status as accord_status};
use crate::protocol::hierarchy::DocumentLocation;
use crate::protocol::review::status as review_status;
use crate::{
    append_event, apply_accord_action, archive_board_document, current_timestamp,
    ensure_file_unchanged, find_board_document, hierarchy_from_workspace, patch_accord_content,
    patch_completion_content, patch_frontmatter_content, read_file_snapshot, split_frontmatter,
    unresolved_blockers, validate_accord_inputs, validate_state,
    validate_task_document_against_hierarchy, validate_task_document_for_mutation, AccordOptions,
    AccordRecord, CliError, Document, HierarchyLock, TandemProject,
};

/// Apply one canonical accord transition and synchronize workflow state.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidationApplyCandidate {
    pub(crate) id: String,
    pub(crate) title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidationActionOutcome {
    pub(crate) id: String,
    pub(crate) state: String,
}

pub(crate) fn accept_validation(
    workspace: &TandemProject,
    id: &str,
    actor: &str,
) -> Result<ValidationActionOutcome, CliError> {
    apply_validation_action(
        workspace,
        id,
        actor,
        ValidationAction::Accept { note: None },
    )
}

pub(crate) fn request_validation_rework(
    workspace: &TandemProject,
    id: &str,
    actor: &str,
    feedback: &str,
) -> Result<ValidationActionOutcome, CliError> {
    let feedback = feedback.trim();
    if feedback.is_empty() {
        return Err(CliError::usage("rework feedback must not be empty"));
    }
    apply_validation_action(
        workspace,
        id,
        actor,
        ValidationAction::Rework {
            feedback: feedback.to_string(),
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidationApplyOutcome {
    pub(crate) completed_ids: Vec<String>,
}

pub(crate) fn accepted_validation_candidates(docs: &[Document]) -> Vec<ValidationApplyCandidate> {
    docs.iter()
        .filter(|doc| doc.doc_type() == "task")
        .filter(|doc| task_state_label(doc) == "validation")
        .filter(|doc| normalize_accord_status(accord_status(doc).unwrap_or("")) == "accepted")
        .filter(|doc| review_status(doc).unwrap_or("") == "accepted")
        .map(|doc| ValidationApplyCandidate {
            id: doc.id().to_string(),
            title: doc.title().to_string(),
        })
        .collect()
}

pub(crate) fn apply_accepted_validation(
    workspace: &TandemProject,
    candidates: &[ValidationApplyCandidate],
    actor: &str,
) -> Result<ValidationApplyOutcome, CliError> {
    if candidates.is_empty() {
        return Err(CliError::usage("no accepted Validation tasks to apply"));
    }
    let _hierarchy_lock = HierarchyLock::acquire(workspace)?;
    hierarchy_from_workspace(workspace)?.validate_all_task_hierarchies()?;
    let mut completed_ids = Vec::new();
    for candidate in candidates {
        complete_validation_candidate(workspace, &candidate.id, actor)?;
        completed_ids.push(candidate.id.clone());
    }
    Ok(ValidationApplyOutcome { completed_ids })
}

fn complete_validation_candidate(
    workspace: &TandemProject,
    id: &str,
    actor: &str,
) -> Result<(), CliError> {
    let doc = find_board_document(workspace, id)?
        .ok_or_else(|| CliError::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can be applied/logged in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    if task_state_label(&doc) != "validation"
        || normalize_accord_status(accord_status(&doc).unwrap_or("")) != "accepted"
        || review_status(&doc).unwrap_or("") != "accepted"
    {
        return Err(CliError::user(format!(
            "{} is not an accepted Validation candidate",
            doc.id()
        )));
    }
    validate_task_document_for_mutation(workspace, &doc)?;
    let unresolved = unresolved_blockers(workspace, doc.field("blockers"))?;
    if !unresolved.is_empty() {
        return Err(CliError::user(format!(
            "Validation failed: {} has unresolved blockers: {}",
            doc.id(),
            unresolved.join(", ")
        )));
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let summary = format!("Applied accepted Validation sign-off for {}", doc.id());
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
        &summary,
        None,
        &[],
        Some("Accepted by Validation apply-accepted workflow"),
        Some(actor),
    )?;
    let _log_path =
        archive_board_document(workspace, &doc.path, &signature, &patched, "completed")?;
    append_event(workspace, "task.completed", doc.id(), &summary)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ValidationAction {
    Accept { note: Option<String> },
    Rework { feedback: String },
}

fn apply_validation_action(
    workspace: &TandemProject,
    id: &str,
    actor: &str,
    action: ValidationAction,
) -> Result<ValidationActionOutcome, CliError> {
    let _hierarchy_lock = HierarchyLock::acquire(workspace)?;
    hierarchy_from_workspace(workspace)?.validate_all_task_hierarchies()?;
    let doc = find_board_document(workspace, id)?
        .ok_or_else(|| CliError::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can use Validation actions in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    if task_state_label(&doc) != "validation" {
        return Err(CliError::user(format!(
            "{} is in `{}`; Validation actions require state `validation`",
            doc.id(),
            task_state_label(&doc)
        )));
    }
    validate_task_document_for_mutation(workspace, &doc)?;

    let previous_status = accord_status(&doc).unwrap_or("missing").to_string();
    if normalize_accord_status(&previous_status) != "delivered" {
        return Err(CliError::user(format!(
            "{} has accord.status={previous_status}; Validation sign-off actions require delivered",
            doc.id()
        )));
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut accord = AccordRecord::from_document(&doc, &now);
    let (
        accord_action,
        status,
        note,
        review_status_value,
        event_name,
        event_summary,
        next_state,
        append_feedback,
    ) = match action {
        ValidationAction::Accept { note } => (
            "accept",
            "accepted",
            note,
            "accepted",
            "validation.accepted",
            format!("Accepted sign-off for {}", doc.id()),
            "validation".to_string(),
            false,
        ),
        ValidationAction::Rework { feedback } => (
            "rework",
            "rework",
            Some(feedback),
            "changes-requested",
            "validation.rework",
            format!("Requested rework for {}", doc.id()),
            "in-progress".to_string(),
            true,
        ),
    };
    let options = AccordOptions {
        id: doc.id().to_string(),
        note: note.clone(),
        reviewer: Some(actor.to_string()),
        ..AccordOptions::default()
    };
    apply_accord_action(&mut accord, accord_action, status, &options);
    let patched = patch_accord_content(&content, &accord)?;
    validate_state(workspace, &next_state)?;
    let mut updates = BTreeMap::new();
    updates.insert("updatedAt".to_string(), now.clone());
    updates.insert("state".to_string(), next_state.clone());
    updates.insert("review.status".to_string(), review_status_value.to_string());
    updates.insert("review.decidedAt".to_string(), now.clone());
    updates.insert("review.reviewer".to_string(), actor.to_string());
    if let Some(note) = note.as_deref().filter(|value| !value.trim().is_empty()) {
        updates.insert("review.note".to_string(), note.to_string());
    }
    let patched = patch_frontmatter_content(&patched, &updates, &[])?;
    let patched = if append_feedback {
        append_feedback_entry(&patched, &now, actor, note.as_deref().unwrap_or(""))?
    } else {
        patched
    };
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(workspace, event_name, doc.id(), &event_summary)?;

    Ok(ValidationActionOutcome {
        id: doc.id().to_string(),
        state: next_state,
    })
}

fn task_state_label(doc: &Document) -> String {
    doc.field("state")
        .filter(|state| !state.trim().is_empty())
        .unwrap_or("unfiled")
        .to_string()
}

fn normalize_accord_status(status: &str) -> String {
    status.trim().to_ascii_lowercase().replace('_', "-")
}

fn append_feedback_entry(
    content: &str,
    timestamp: &str,
    source: &str,
    feedback: &str,
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let mut body = body.to_string();
    if !body.ends_with('\n') {
        body.push('\n');
    }
    if !body.contains("\n## Feedback\n") && !body.trim_start().starts_with("## Feedback\n") {
        if !body.trim().is_empty() {
            body.push('\n');
        }
        body.push_str("## Feedback\n\n");
    } else if !body.ends_with("\n\n") {
        body.push('\n');
    }
    body.push_str(&format!(
        "- {timestamp} ({source}): {}\n",
        feedback.replace('\n', " ").trim()
    ));
    Ok(format!("---\n{}---\n{}", frontmatter, body))
}

#[derive(Debug)]
pub(crate) struct AccordTransitionOutcome {
    pub(crate) id: String,
    pub(crate) previous_status: String,
    pub(crate) status: String,
    pub(crate) previous_state: String,
    pub(crate) synced_state: Option<String>,
    pub(crate) event_name: String,
    pub(crate) path: PathBuf,
}

pub(crate) fn transition(
    workspace: &TandemProject,
    action: &str,
    options: AccordOptions,
) -> Result<AccordTransitionOutcome, CliError> {
    let status = accord::status_for_action(action)
        .ok_or_else(|| CliError::usage(format!("unknown accord action `{action}`")))?;
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    let hierarchy = hierarchy_from_workspace(workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can have accord actions in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;
    validate_accord_inputs(action, &options)?;
    let previous_status = accord_status(&doc).unwrap_or("missing").to_string();
    accord::validate_transition(action, &previous_status).map_err(CliError::user)?;
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut accord = AccordRecord::from_document(&doc, &now);
    apply_accord_action(&mut accord, action, status, &options);
    let patched = patch_accord_content(&content, &accord)?;
    let mut updates = BTreeMap::new();
    updates.insert("updatedAt".to_string(), now);
    let previous_state = doc.field("state").unwrap_or("-").to_string();
    let synced_state = accord::state_sync_target(status, &previous_state).map(str::to_string);
    if let Some(state) = synced_state.as_deref() {
        validate_state(workspace, state)?;
        updates.insert("state".to_string(), state.to_string());
    }
    let patched = patch_frontmatter_content(&patched, &updates, &[])?;
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    let event_name = accord::event_name(action).to_string();
    append_event(
        workspace,
        &event_name,
        doc.id(),
        &format!("Accord {action} for {}", doc.id()),
    )?;
    Ok(AccordTransitionOutcome {
        id: doc.id().to_string(),
        previous_status,
        status: status.to_string(),
        previous_state,
        synced_state,
        event_name,
        path: doc.path,
    })
}
