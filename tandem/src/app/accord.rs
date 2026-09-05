//! Shared accord and Validation lifecycle operations.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::support::{
    append_event, checkpoint_boundary, current_timestamp,
    hierarchy_from_project as hierarchy_from_workspace, require_nonempty, validate_state,
    validate_task_document_against_hierarchy,
};
use crate::app::Error;
use crate::project::write::{ensure_file_unchanged, read_file_snapshot, HierarchyLock};
use crate::project::{
    self, patch_accord_content, patch_frontmatter_content, patch_resolution_content,
    split_frontmatter, write_atomic, CheckpointOutcome, StoredDocument as Document, TandemProject,
};
use crate::protocol::accord::{self, status as accord_status, AccordRecord};
use crate::protocol::hierarchy::DocumentLocation;
use crate::protocol::workflow::{
    ResolutionRecord, RESOLUTION_OUTCOME_COMPLETED, RESOLUTION_OUTCOME_FAILED,
};

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

fn validate_accord_inputs(action: &str, options: &AccordOptions) -> Result<(), Error> {
    let requirement = match action {
        "claim" => Some((
            options.assignee.as_deref(),
            "accord claim requires --assignee <name>".to_string(),
        )),
        "deliver" => Some((
            options.summary.as_deref(),
            "accord deliver requires --summary <text>".to_string(),
        )),
        "rework" | "block" | "release" | "fail" => Some((
            options.note.as_deref().or(options.reason.as_deref()),
            format!("accord {action} requires --note <text>"),
        )),
        _ => None,
    };
    if let Some((value, message)) = requirement {
        require_nonempty(value, &message)?;
    }
    if action == "deliver" {
        accord::validate_delivery_evidence(&options.evidence).map_err(Error::usage)?;
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
        "rework" => {
            accord.reviewer = None;
            accord.reason = None;
        }
        "block" | "fail" | "release" => {
            accord.reviewer = None;
            accord.note = None;
        }
        "resume" => {
            accord.note = None;
            accord.reason = None;
        }
        _ => {}
    }
    if let Some(value) = options
        .note
        .as_deref()
        .or(options.reason.as_deref())
        .filter(|v| !v.trim().is_empty())
    {
        accord.note = Some(value.to_string());
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
pub(crate) struct ValidationActionOutcome {
    pub(crate) id: String,
    pub(crate) state: String,
    pub(crate) checkpoint: CheckpointOutcome,
}

pub(crate) fn accept_validation(
    workspace: &TandemProject,
    id: &str,
    actor: &str,
) -> Result<ValidationActionOutcome, Error> {
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
) -> Result<ValidationActionOutcome, Error> {
    let feedback = feedback.trim();
    if feedback.is_empty() {
        return Err(Error::usage("rework feedback must not be empty"));
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
enum ValidationAction {
    Accept { note: Option<String> },
    Rework { feedback: String },
}

fn apply_validation_action(
    workspace: &TandemProject,
    id: &str,
    actor: &str,
    action: ValidationAction,
) -> Result<ValidationActionOutcome, Error> {
    let _hierarchy_lock = HierarchyLock::acquire(workspace)?;
    let hierarchy = hierarchy_from_workspace(workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let doc = hierarchy
        .document(id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| Error::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(Error::user(format!(
            "Validation failed: only task documents can use Validation actions in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    if task_state_label(&doc) != "validation" {
        return Err(Error::user(format!(
            "{} is in `{}`; Validation actions require state `validation`",
            doc.id(),
            task_state_label(&doc)
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;

    let previous_status = accord_status(&doc).unwrap_or("missing").to_string();
    if normalize_accord_status(&previous_status) != "delivered" {
        return Err(Error::user(format!(
            "{} has accord.status={previous_status}; Validation sign-off actions require delivered",
            doc.id()
        )));
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut accord = AccordRecord::from_document(&doc, &now);
    match action {
        ValidationAction::Accept { note } => {
            // Protocol 0.3.0 (D16/D18/D39): human acceptance atomically
            // accepts the delivered Accord and archives the Task. There is no
            // accepted-but-active state.
            let options = AccordOptions {
                id: doc.id().to_string(),
                note: note.clone(),
                reviewer: Some(actor.to_string()),
                ..AccordOptions::default()
            };
            apply_accord_action(&mut accord, "accept", "accepted", &options);
            let patched = patch_accord_content(&content, &accord)?;
            let mut updates = BTreeMap::new();
            updates.insert("updatedAt".to_string(), now.clone());
            updates.insert("archivedAt".to_string(), now.clone());
            let patched = patch_frontmatter_content(
                &patched,
                &updates,
                &[
                    "state",
                    "completedAt",
                    "completionSummary",
                    "completionValidation",
                    "completionReviewer",
                    "filesChanged",
                    "validation.state",
                    "validation.criterion",
                    "validation.note",
                    "validation.reviewer",
                    "validation.requestedAt",
                    "review",
                ],
            )?;
            let note_text = note.as_deref().unwrap_or("Accepted by human validation");
            let patched = patch_resolution_content(
                &patched,
                &ResolutionRecord {
                    outcome: Some(RESOLUTION_OUTCOME_COMPLETED.to_string()),
                    note: Some(note_text.to_string()),
                    reviewer: Some(actor.to_string()),
                },
            )?;
            project::write::archive_board_document(
                workspace,
                &doc.path,
                &signature,
                &patched,
                "completed",
            )?;
            append_event(
                workspace,
                "validation.accepted",
                doc.id(),
                &format!("Accepted sign-off for {}", doc.id()),
            )?;
            let checkpoint = checkpoint_boundary(workspace);
            Ok(ValidationActionOutcome {
                id: doc.id().to_string(),
                state: "archived".to_string(),
                checkpoint,
            })
        }
        ValidationAction::Rework { feedback } => {
            let options = AccordOptions {
                id: doc.id().to_string(),
                note: Some(feedback.clone()),
                reviewer: Some(actor.to_string()),
                ..AccordOptions::default()
            };
            apply_accord_action(&mut accord, "rework", "rework", &options);
            let patched = patch_accord_content(&content, &accord)?;
            validate_state(workspace, "in-progress")?;
            let mut updates = BTreeMap::new();
            updates.insert("updatedAt".to_string(), now.clone());
            updates.insert("state".to_string(), "in-progress".to_string());
            let patched = patch_frontmatter_content(
                &patched,
                &updates,
                &[
                    "validation.state",
                    "validation.criterion",
                    "validation.note",
                    "validation.reviewer",
                    "validation.requestedAt",
                    "review",
                ],
            )?;
            let patched = append_feedback_entry(&patched, &now, actor, &feedback)?;
            ensure_file_unchanged(&doc.path, &signature)?;
            write_atomic(&doc.path, &patched)?;
            append_event(
                workspace,
                "validation.rework",
                doc.id(),
                &format!("Requested rework for {}", doc.id()),
            )?;
            let checkpoint = checkpoint_boundary(workspace);
            Ok(ValidationActionOutcome {
                id: doc.id().to_string(),
                state: "in-progress".to_string(),
                checkpoint,
            })
        }
    }
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
) -> Result<String, Error> {
    let (frontmatter, body) = split_frontmatter(content).map_err(Error::user)?;
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
    pub(crate) checkpoint: CheckpointOutcome,
}

pub(crate) fn transition(
    workspace: &TandemProject,
    action: &str,
    options: AccordOptions,
) -> Result<AccordTransitionOutcome, Error> {
    let status = accord::status_for_action(action)
        .ok_or_else(|| Error::usage(format!("unknown accord action `{action}`")))?;
    let _hierarchy_lock = project::write::HierarchyLock::acquire(workspace)?;
    let hierarchy = hierarchy_from_workspace(workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| Error::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(Error::user(format!(
            "Validation failed: only task documents can have accord actions in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;
    validate_accord_inputs(action, &options)?;
    let previous_status = accord_status(&doc).unwrap_or("missing").to_string();
    accord::validate_transition(action, &previous_status).map_err(Error::user)?;
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut accord = AccordRecord::from_document(&doc, &now);
    apply_accord_action(&mut accord, action, status, &options);
    let patched = patch_accord_content(&content, &accord)?;
    let mut updates = BTreeMap::new();
    updates.insert("updatedAt".to_string(), now.clone());
    // Protocol 0.3.0 (D23): ownership is the top-level assignee. Claim sets it
    // and release clears it; there is no accord-nested copy.
    let mut assignee_removes: Vec<&str> = Vec::new();
    if action == "release" {
        assignee_removes.push("assignee");
    } else if let Some(value) = options.assignee.as_deref().filter(|v| !v.trim().is_empty()) {
        updates.insert("assignee".to_string(), value.to_string());
    }
    let previous_state = doc.field("state").unwrap_or("-").to_string();
    let effect = accord::state_effect(action, &previous_state);
    let synced_state = if let Some(state) = effect.state {
        validate_state(workspace, state)?;
        updates.insert("state".to_string(), state.to_string());
        Some(state.to_string())
    } else {
        None
    };
    let mut removes = assignee_removes;
    if action == "rework" {
        removes.extend([
            "validation.state",
            "validation.criterion",
            "validation.note",
            "validation.reviewer",
            "validation.requestedAt",
        ]);
    }
    let patched = patch_frontmatter_content(&patched, &updates, &removes)?;
    // Protocol 0.3.0 (D21): fail means the agreed outcome cannot be achieved;
    // it atomically archives the Task as failed with the transition note.
    if action == "fail" {
        let reason = options
            .note
            .as_deref()
            .or(options.reason.as_deref())
            .unwrap_or("accord failed");
        let mut fail_updates = BTreeMap::new();
        fail_updates.insert("updatedAt".to_string(), now.clone());
        fail_updates.insert("archivedAt".to_string(), now);
        let patched = patch_frontmatter_content(
            &patched,
            &fail_updates,
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
                outcome: Some(RESOLUTION_OUTCOME_FAILED.to_string()),
                note: Some(reason.to_string()),
                reviewer: options.reviewer.clone(),
            },
        )?;
        let log_path = project::write::archive_board_document(
            workspace, &doc.path, &signature, &patched, "failed",
        )?;
        append_event(
            workspace,
            &accord::event_name(action),
            doc.id(),
            &format!("Accord {action} for {}", doc.id()),
        )?;
        let checkpoint = checkpoint_boundary(workspace);
        return Ok(AccordTransitionOutcome {
            id: doc.id().to_string(),
            previous_status,
            status: status.to_string(),
            previous_state,
            synced_state: None,
            event_name: accord::event_name(action).to_string(),
            path: log_path,
            checkpoint,
        });
    }

    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    let event_name = accord::event_name(action).to_string();
    append_event(
        workspace,
        &event_name,
        doc.id(),
        &format!("Accord {action} for {}", doc.id()),
    )?;
    let checkpoint = checkpoint_boundary(workspace);

    Ok(AccordTransitionOutcome {
        id: doc.id().to_string(),
        previous_status,
        status: status.to_string(),
        previous_state,
        synced_state,
        event_name,
        path: doc.path,
        checkpoint,
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
    fn deliver_rejects_empty_evidence_without_mutating_record_or_events() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let root = std::env::temp_dir().join(format!(
            "tandem-app-delivery-evidence-{}",
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
        let created = crate::app::tasks::add(
            &project,
            crate::app::tasks::AddOptions {
                title: Some("Evidence task".to_string()),
                acceptance: vec!["acceptance".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        transition(
            &project,
            "claim",
            AccordOptions {
                id: created.id.clone(),
                assignee: Some("worker".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let path = created.path;
        let before = fs::read_to_string(&path).unwrap();
        let events_before = project.read_events_tolerant(&mut Vec::new()).len();
        for evidence in [Vec::new(), vec![String::new()], vec!["  \n\t".to_string()]] {
            let error = transition(
                &project,
                "deliver",
                AccordOptions {
                    id: created.id.clone(),
                    summary: Some("observed".to_string()),
                    evidence,
                    ..Default::default()
                },
            )
            .unwrap_err();
            assert!(error.message.contains("non-empty --evidence"));
            assert_eq!(fs::read_to_string(&path).unwrap(), before);
            assert_eq!(
                project.read_events_tolerant(&mut Vec::new()).len(),
                events_before
            );
        }
        transition(
            &project,
            "deliver",
            AccordOptions {
                id: created.id,
                summary: Some("observed".to_string()),
                evidence: vec!["observed output".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("status: \"delivered\""));
        assert!(after.contains("observed output"));
        assert_eq!(
            project.read_events_tolerant(&mut Vec::new()).len(),
            events_before + 1
        );
        fs::remove_dir_all(project.root()).unwrap();
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
        let path = project.tasks_dir.join("task-1.md");
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
        assert_eq!(delivered.synced_state, None);

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
    fn accord_definition_fields_survive_every_transition() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let root = std::env::temp_dir().join(format!(
            "tandem-app-acceptance-{}",
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
        let path = project.tasks_dir.join("task-1.md");
        fs::write(
            &path,
            "---\nid: task-1\ntype: task\ntitle: Durable accord\nstate: todo\naccord:\n  status: ready\n  acceptance: [\"criterion one\", \"criterion two\"]\n  constraints: [\"no new deps\"]\n---\n# Body\n",
        )
        .unwrap();

        let assert_definition_intact = |label: &str| {
            let content = fs::read_to_string(&path).unwrap();
            assert!(
                content.contains("criterion one") && content.contains("criterion two"),
                "acceptance criteria lost after {label}: {content}"
            );
            assert!(
                content.contains("no new deps"),
                "constraints lost after {label}: {content}"
            );
        };

        let options = |extra: AccordOptions| AccordOptions {
            id: "task-1".to_string(),
            ..extra
        };

        transition(
            &project,
            "claim",
            options(AccordOptions {
                assignee: Some("worker-a".to_string()),
                ..AccordOptions::default()
            }),
        )
        .unwrap();
        assert_definition_intact("claim");

        transition(
            &project,
            "block",
            options(AccordOptions {
                note: Some("waiting".to_string()),
                ..AccordOptions::default()
            }),
        )
        .unwrap();
        assert_definition_intact("block");

        transition(&project, "resume", options(AccordOptions::default())).unwrap();
        assert_definition_intact("resume");

        transition(
            &project,
            "deliver",
            options(AccordOptions {
                summary: Some("Ready".to_string()),
                evidence: vec!["tests pass".to_string()],
                ..AccordOptions::default()
            }),
        )
        .unwrap();
        assert_definition_intact("deliver");

        transition(
            &project,
            "rework",
            options(AccordOptions {
                note: Some("fix it".to_string()),
                ..AccordOptions::default()
            }),
        )
        .unwrap();
        assert_definition_intact("rework");

        transition(
            &project,
            "release",
            options(AccordOptions {
                note: Some("reassigning".to_string()),
                ..AccordOptions::default()
            }),
        )
        .unwrap();
        assert_definition_intact("release");

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
        let path = project.tasks_dir.join("task-1.md");
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
