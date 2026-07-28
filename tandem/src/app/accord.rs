//! Shared accord and Validation lifecycle operations.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::support::{
    append_event, current_timestamp, hierarchy_from_project as hierarchy_from_workspace,
    require_nonempty, validate_state, validate_task_document_against_hierarchy,
};
use crate::project::write::{
    archive_board_document, ensure_file_unchanged, read_file_snapshot, HierarchyLock,
};
use crate::project::{
    self, patch_accord_content, patch_completion_content, patch_frontmatter_content,
    split_frontmatter, write_atomic, StoredDocument as Document, TandemProject,
};
use crate::protocol::accord::{self, status as accord_status, AccordRecord};
use crate::protocol::hierarchy::DocumentLocation;
use crate::protocol::review::status as review_status;
use crate::protocol::workflow::CompletionRecord;
use crate::CliError;

#[derive(Debug, Default)]
pub(crate) struct AccordOptions {
    pub(crate) id: String,
    pub(crate) assignee: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) reviewer: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) reason: Option<String>,
    pub(crate) deliverables: Vec<String>,
    pub(crate) validations: Vec<String>,
    pub(crate) constraints: Vec<String>,
    pub(crate) evidence: Vec<String>,
    pub(crate) files_changed: Vec<String>,
}

fn validate_accord_inputs(action: &str, options: &AccordOptions) -> Result<(), CliError> {
    let requirement = match action {
        "claim" => Some((
            options.assignee.as_deref(),
            "accord claim requires --assignee <name>".to_string(),
        )),
        "deliver" => Some((
            options.summary.as_deref(),
            "accord deliver requires --summary <text>".to_string(),
        )),
        "rework" => Some((
            options.note.as_deref(),
            "accord rework requires --note <text>".to_string(),
        )),
        "block" | "fail" => Some((
            options.reason.as_deref(),
            format!("accord {action} requires --reason <text>"),
        )),
        _ => None,
    };
    if let Some((value, message)) = requirement {
        require_nonempty(value, &message)?;
    }
    Ok(())
}

fn apply_accord_action(
    accord: &mut AccordRecord,
    action: &str,
    status: &str,
    options: &AccordOptions,
) {
    accord.status = status.to_string();
    match action {
        "claim" => {
            accord.claimed_at = Some(accord.updated_at.clone());
            accord.delivered_at = None;
            accord.summary = None;
            accord.evidence.clear();
            accord.files_changed.clear();
            accord.reviewer = None;
            accord.note = None;
            accord.reason = None;
        }
        "deliver" => {
            accord.delivered_at = Some(accord.updated_at.clone());
            accord.reviewer = None;
            accord.note = None;
            accord.reason = None;
        }
        "accept" => accord.reason = None,
        "rework" => {
            accord.reviewer = None;
            accord.reason = None;
        }
        "block" | "fail" => {
            accord.reviewer = None;
            accord.note = None;
        }
        _ => {}
    }
    if let Some(value) = options.assignee.as_deref().filter(|v| !v.trim().is_empty()) {
        accord.assignee = Some(value.to_string());
    }
    if !options.deliverables.is_empty() {
        accord.deliverables.clone_from(&options.deliverables);
    }
    if !options.validations.is_empty() {
        accord.validations.clone_from(&options.validations);
    }
    if !options.constraints.is_empty() {
        accord.constraints.clone_from(&options.constraints);
    }
    if let Some(value) = options.summary.as_deref().filter(|v| !v.trim().is_empty()) {
        accord.summary = Some(value.to_string());
    }
    if !options.evidence.is_empty() {
        accord.evidence.clone_from(&options.evidence);
    }
    if !options.files_changed.is_empty() {
        accord.files_changed.clone_from(&options.files_changed);
    }
    if let Some(value) = options.reviewer.as_deref().filter(|v| !v.trim().is_empty()) {
        accord.reviewer = Some(value.to_string());
    }
    if let Some(value) = options.note.as_deref().filter(|v| !v.trim().is_empty()) {
        accord.note = Some(value.to_string());
    }
    if let Some(value) = options.reason.as_deref().filter(|v| !v.trim().is_empty()) {
        accord.reason = Some(value.to_string());
    }
}

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
    let hierarchy = hierarchy_from_workspace(workspace)?;
    let doc = hierarchy
        .document(id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
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
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;
    let unresolved =
        crate::app::support::unresolved_blockers_in_hierarchy(&hierarchy, doc.field("blockers"));
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
        &CompletionRecord {
            summary: summary.clone(),
            validation: Some("Accepted by Validation apply-accepted workflow".to_string()),
            reviewer: Some(actor.to_string()),
            ..CompletionRecord::default()
        },
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
    let hierarchy = hierarchy_from_workspace(workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let doc = hierarchy
        .document(id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
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
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_entry_preserves_caller_actor_and_body() {
        let content = "---\nid: task-1\n---\n# Delivery\n";
        let patched = append_feedback_entry(content, "now", "review-bot", "Fix\ncontrast").unwrap();
        assert!(patched.contains("# Delivery"));
        assert!(patched.contains("- now (review-bot): Fix contrast"));
    }

    #[test]
    fn accord_status_normalization_remains_protocol_shaped() {
        assert_eq!(normalize_accord_status(" Delivered "), "delivered");
        assert_eq!(
            normalize_accord_status("changes_requested"),
            "changes-requested"
        );
    }

    #[test]
    fn accord_transition_preserves_metadata_syncs_state_and_appends_events() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let root = std::env::temp_dir().join(format!(
            "tandem-app-accord-{}",
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
        let path = project.board_dir.join("task-1.md");
        fs::write(
            &path,
            "---\nid: task-1\ntype: task\ntitle: Accord task\nstate: todo\nunknown: keep\n---\n# Body\n",
        )
        .unwrap();

        let claimed = transition(
            &project,
            "claim",
            AccordOptions {
                id: "task-1".to_string(),
                assignee: Some("worker-a".to_string()),
                ..AccordOptions::default()
            },
        )
        .unwrap();
        assert_eq!(claimed.previous_status, "missing");
        assert_eq!(claimed.synced_state.as_deref(), Some("in-progress"));

        let delivered = transition(
            &project,
            "deliver",
            AccordOptions {
                id: "task-1".to_string(),
                summary: Some("Ready".to_string()),
                evidence: vec!["tests pass".to_string()],
                ..AccordOptions::default()
            },
        )
        .unwrap();
        assert_eq!(delivered.previous_status, "claimed");
        assert_eq!(delivered.synced_state.as_deref(), Some("validation"));

        let changed = fs::read_to_string(path).unwrap();
        assert!(changed.contains("unknown: keep"));
        assert!(changed.contains("# Body"));
        assert!(changed.contains("status: \"delivered\""));
        assert!(changed.contains("assignee: \"worker-a\""));
        assert!(changed.contains("summary: \"Ready\""));
        let events = project.read_events_tolerant(&mut Vec::new());
        assert!(events.iter().any(|event| event.event == "accord.claimed"));
        assert!(events.iter().any(|event| event.event == "accord.delivered"));
        fs::remove_dir_all(project.root()).unwrap();
    }

    #[test]
    fn apply_accepted_validation_archives_with_actor_unknown_fields_and_event() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let root = std::env::temp_dir().join(format!(
            "tandem-app-validation-apply-{}",
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
        fs::write(
            project.board_dir.join("task-1.md"),
            "---\nid: task-1\ntype: task\ntitle: Accepted\nstate: validation\ncustom: keep\naccord:\n  status: accepted\nreview:\n  status: accepted\n---\n# Body\n",
        )
        .unwrap();

        let outcome = apply_accepted_validation(
            &project,
            &[ValidationApplyCandidate {
                id: "task-1".to_string(),
                title: "Accepted".to_string(),
            }],
            "human-reviewer",
        )
        .unwrap();
        assert_eq!(outcome.completed_ids, vec!["task-1"]);
        assert!(!project.board_dir.join("task-1.md").exists());
        let archived = fs::read_to_string(project.logs_dir.join("task-1.md")).unwrap();
        assert!(archived.contains("custom: keep"));
        assert!(archived.contains("# Body"));
        assert!(archived.contains("reviewer: \"human-reviewer\""));
        assert!(archived.contains("validation: \"Accepted by Validation apply-accepted workflow\""));
        let events = project.read_events_tolerant(&mut Vec::new());
        assert!(events.iter().any(|event| event.event == "task.completed"));
        fs::remove_dir_all(project.root()).unwrap();
    }

    #[test]
    fn validation_rework_uses_supplied_actor_in_document_and_event() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let root = std::env::temp_dir().join(format!(
            "tandem-app-validation-{}",
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
        let path = project.board_dir.join("task-1.md");
        fs::write(
            &path,
            "---\nid: task-1\ntype: task\ntitle: Delivered\nstate: validation\naccord:\n  status: delivered\n---\n# Body\n",
        )
        .unwrap();

        let outcome =
            request_validation_rework(&project, "task-1", "review-bot", "Fix contrast").unwrap();
        assert_eq!(outcome.state, "in-progress");
        let changed = fs::read_to_string(path).unwrap();
        assert!(changed.contains("reviewer: \"review-bot\""));
        assert!(changed.contains("(review-bot): Fix contrast"));
        let events = project.read_events_tolerant(&mut Vec::new());
        assert!(events
            .iter()
            .any(|event| event.event == "validation.rework"));
        fs::remove_dir_all(project.root()).unwrap();
    }
}
