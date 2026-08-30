//! Canonical structural diagnostic severity and document-level checks.

use super::accord;
use super::document::{has_metadata, Document};
use super::workflow::{resolution_note, resolution_outcome, RESOLUTION_OUTCOMES};

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

pub(crate) fn completion_policy_diagnostics(document: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let status = accord::status(document)
        .unwrap_or("missing")
        .to_ascii_lowercase();
    // D16: completing a delivered Task accepts the Accord atomically, so only
    // a status that is neither delivered nor accepted deserves a warning.
    if !matches!(status.as_str(), "accepted" | "delivered") {
        diagnostics.push(Diagnostic::warning(format!(
            "{} has accord.status={status}; complete normally follows a delivered Accord.",
            document.id()
        )));
    }
    diagnostics
}
