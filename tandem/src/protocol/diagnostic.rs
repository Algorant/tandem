//! Canonical structural diagnostic severity and document-level checks.

use super::accord;
use super::document::{has_metadata, Document};
use super::hierarchy::{DocumentLocation, TaskRole};
use super::workflow::{
    resolution_note, resolution_outcome, RESOLUTION_OUTCOMES, RESOLUTION_OUTCOME_COMPLETED,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Diagnostic {
    pub(crate) severity: Severity,
    pub(crate) message: String,
}

impl Diagnostic {
    pub(crate) fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
        }
    }

    pub(crate) fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
        }
    }
}

pub(crate) fn metadata_diagnostics(document: &Document, is_log: bool) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if document.id().trim().is_empty() {
        diagnostics.push(Diagnostic::error("missing required field `id`"));
    }
    if document.title().trim().is_empty() {
        diagnostics.push(Diagnostic::error("missing required field `title`"));
    }
    if document
        .field("type")
        .is_none_or(|value| value.trim().is_empty())
    {
        diagnostics.push(Diagnostic::error("missing required field `type`"));
    }
    if is_log && document.doc_type() == "task" {
        if document.field("archivedAt").is_none() {
            diagnostics.push(Diagnostic::error("missing required log field `archivedAt`"));
        }
        if resolution_note(document).is_none() {
            diagnostics.push(Diagnostic::error(
                "missing required log field `resolution.note`",
            ));
        }
        let outcome = resolution_outcome(document);
        if !RESOLUTION_OUTCOMES.contains(&outcome) {
            diagnostics.push(Diagnostic::error(format!(
                "invalid resolution.outcome `{outcome}`; expected completed, canceled, or failed"
            )));
        }
    }
    if has_metadata(document, "accord") || document.field("accordStatus").is_some() {
        match accord::status(document) {
            Some(status) if accord::is_known_status(status) => {}
            Some(status) => diagnostics.push(Diagnostic::error(format!(
                "invalid accord.status `{status}`"
            ))),
            None => diagnostics.push(Diagnostic::error(
                "accord.status is required when accord metadata is present",
            )),
        }
    }
    if has_metadata(document, "review") || document.field("reviewStatus").is_some() {
        diagnostics.push(Diagnostic::warning(format!(
            "{} carries legacy review metadata; protocol 0.3.0 uses state=validation and validation.criterion/note instead.",
            document.id()
        )));
    }
    diagnostics
}

pub(crate) fn workflow_state_diagnostic(
    document: &Document,
    is_active_task: bool,
    states: &[String],
) -> Option<Diagnostic> {
    if !is_active_task {
        return None;
    }
    match document.field("state") {
        Some(state) if states.iter().any(|known| known == state) => None,
        Some(state) if !state.trim().is_empty() => Some(Diagnostic::error(format!(
            "unknown state `{state}`; known states: {}",
            states.join(", ")
        ))),
        _ => Some(Diagnostic::error("missing required field `state`")),
    }
}

/// Resolved placement and canonical archive outcome of one descendant in the
/// hierarchy of the document being completed. The app assembles these facts
/// from one coherent Board-and-Logs snapshot; this module owns the completion
/// policy that consumes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedDescendant<'a> {
    pub(crate) location: DocumentLocation,
    /// Explicit canonical `resolution.outcome`, if the archived record carries
    /// it. Absent or legacy-only records stay `None` so they never count as
    /// positive completion evidence.
    pub(crate) canonical_outcome: Option<&'a str>,
}

/// The canonical archive outcome: only the explicit `resolution.outcome`
/// field. Unlike [`resolution_outcome`], this never falls back to legacy
/// `completion.*` fields or to an implicit `completed`, so policy that needs
/// positive evidence can distinguish absent, legacy, and malformed records.
pub(crate) fn canonical_resolution_outcome(document: &Document) -> Option<&str> {
    document.field("resolution.outcome")
}

pub(crate) fn completion_policy_diagnostics(
    document: &Document,
    role: Option<TaskRole>,
    descendants: &[ResolvedDescendant<'_>],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let status = accord::status(document)
        .unwrap_or("missing")
        .to_ascii_lowercase();
    // D16: completing a delivered Task accepts the Accord atomically, so only
    // a status that is neither delivered nor accepted deserves a warning. The
    // child-based Epic exception below is the only suppression: it never
    // fabricates a delivery or acceptance for the grouping Epic.
    if !matches!(status.as_str(), "accepted" | "delivered")
        && !epic_child_hierarchy_completed(role, descendants)
    {
        diagnostics.push(Diagnostic::warning(format!(
            "{} has accord.status={status}; complete normally follows a delivered Accord.",
            document.id()
        )));
    }
    diagnostics
}

/// Child-based Epic closure: a resolved Epic with at least one descendant
/// qualifies only when every descendant is archived with an explicit canonical
/// `resolution.outcome` of `completed`. Empty hierarchies, active Board
/// descendants, absent/legacy/unknown outcomes, and canceled or failed
/// children all retain the ordinary missing-delivery warning.
fn epic_child_hierarchy_completed(
    role: Option<TaskRole>,
    descendants: &[ResolvedDescendant<'_>],
) -> bool {
    role == Some(TaskRole::Epic)
        && !descendants.is_empty()
        && descendants.iter().all(|descendant| {
            descendant.location == DocumentLocation::Logs
                && descendant.canonical_outcome == Some(RESOLUTION_OUTCOME_COMPLETED)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn task(id: &str, accord_status: &str) -> Document {
        Document::new(
            HashMap::from([
                ("id".to_string(), id.to_string()),
                ("type".to_string(), "task".to_string()),
                ("title".to_string(), id.to_string()),
                ("accord.status".to_string(), accord_status.to_string()),
            ]),
            String::new(),
        )
    }

    fn archived(outcome: Option<&str>) -> ResolvedDescendant<'_> {
        ResolvedDescendant {
            location: DocumentLocation::Logs,
            canonical_outcome: outcome,
        }
    }

    fn active() -> ResolvedDescendant<'static> {
        ResolvedDescendant {
            location: DocumentLocation::Board,
            canonical_outcome: None,
        }
    }

    fn message(
        document: &Document,
        role: Option<TaskRole>,
        descendants: &[ResolvedDescendant<'_>],
    ) -> Option<String> {
        completion_policy_diagnostics(document, role, descendants)
            .into_iter()
            .next()
            .map(|diagnostic| diagnostic.message)
    }

    #[test]
    fn ready_epic_closes_without_warning_for_all_completed_archived_children() {
        let epic = task("task-1", "ready");
        let descendants = [archived(Some("completed")), archived(Some("completed"))];
        assert_eq!(message(&epic, Some(TaskRole::Epic), &descendants), None);
    }

    #[test]
    fn empty_epic_retains_the_delivery_warning() {
        let epic = task("task-1", "ready");
        let warning = message(&epic, Some(TaskRole::Epic), &[]).expect("warning");
        assert!(warning.contains("complete normally follows a delivered Accord"));
    }

    #[test]
    fn active_board_descendant_retains_the_delivery_warning() {
        let epic = task("task-1", "ready");
        let descendants = [archived(Some("completed")), active()];
        assert!(message(&epic, Some(TaskRole::Epic), &descendants)
            .expect("warning")
            .contains("complete normally follows"));
    }

    #[test]
    fn absent_canonical_outcome_never_counts_as_completed() {
        let epic = task("task-1", "ready");
        let descendants = [archived(Some("completed")), archived(None)];
        assert!(message(&epic, Some(TaskRole::Epic), &descendants)
            .expect("warning")
            .contains("complete normally follows"));
    }

    #[test]
    fn unknown_canonical_outcome_never_counts_as_completed() {
        let epic = task("task-1", "ready");
        let descendants = [archived(Some("bogus"))];
        assert!(message(&epic, Some(TaskRole::Epic), &descendants)
            .expect("warning")
            .contains("complete normally follows"));
    }

    #[test]
    fn canceled_and_failed_children_retain_the_delivery_warning() {
        let epic = task("task-1", "ready");
        for outcome in ["canceled", "failed"] {
            let descendants = [archived(Some("completed")), archived(Some(outcome))];
            assert!(
                message(&epic, Some(TaskRole::Epic), &descendants)
                    .expect("warning")
                    .contains("complete normally follows"),
                "{outcome} must retain the warning"
            );
        }
    }

    #[test]
    fn ordinary_task_never_uses_the_epic_exception() {
        let ordinary = task("task-1", "ready");
        let descendants = [archived(Some("completed"))];
        assert!(message(&ordinary, Some(TaskRole::Task), &descendants)
            .expect("warning")
            .contains("complete normally follows"));
    }

    #[test]
    fn explicitly_delivered_or_accepted_documents_stay_warning_free() {
        for status in ["delivered", "accepted"] {
            let document = task("task-1", status);
            assert_eq!(
                message(&document, Some(TaskRole::Epic), &[]),
                None,
                "{status} stays warning-free"
            );
        }
    }
}
