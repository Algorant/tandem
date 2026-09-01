//! Canonical accord vocabulary, transitions, and workflow alignment.

use super::document::{parse_field_values, Document};

#[derive(Debug, Clone, Default)]
pub(crate) struct AccordRecord {
    pub(crate) status: String,
    pub(crate) acceptance: Vec<String>,
    pub(crate) claimed_at: Option<String>,
    pub(crate) delivered_at: Option<String>,
    pub(crate) deliverables: Vec<String>,
    pub(crate) validations: Vec<String>,
    pub(crate) constraints: Vec<String>,
    pub(crate) summary: Option<String>,
    pub(crate) evidence: Vec<String>,
    pub(crate) files_changed: Vec<String>,
    pub(crate) reviewer: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) reason: Option<String>,
    pub(crate) updated_at: String,
}

impl AccordRecord {
    pub(crate) fn from_document(doc: &Document, updated_at: &str) -> Self {
        Self {
            status: status(doc).unwrap_or("missing").to_string(),
            acceptance: doc
                .field("accord.acceptance")
                .map(parse_field_values)
                .unwrap_or_default(),
            claimed_at: doc.field("accord.claimedAt").map(str::to_string),
            delivered_at: doc.field("accord.deliveredAt").map(str::to_string),
            deliverables: doc
                .field("accord.deliverables")
                .map(parse_field_values)
                .unwrap_or_default(),
            validations: doc
                .field("accord.validation")
                .map(parse_field_values)
                .unwrap_or_default(),
            constraints: doc
                .field("accord.constraints")
                .map(parse_field_values)
                .unwrap_or_default(),
            summary: doc.field("accord.summary").map(str::to_string),
            evidence: doc
                .field("accord.evidence")
                .map(parse_field_values)
                .unwrap_or_default(),
            files_changed: doc
                .field("accord.filesChanged")
                .map(parse_field_values)
                .unwrap_or_default(),
            reviewer: doc.field("accord.reviewer").map(str::to_string),
            note: doc.field("accord.note").map(str::to_string),
            reason: doc.field("accord.reason").map(str::to_string),
            updated_at: updated_at.to_string(),
        }
    }
}

pub(crate) const STATUSES: &[&str] = &[
    "ready",
    "claimed",
    "delivered",
    "accepted",
    "rework",
    "failed",
    "blocked",
];
pub(crate) const LEGACY_STATUSES: &[&str] = &[];
pub(crate) const ACTIONS: &[&str] = &[
    "claim", "deliver", "rework", "block", "resume", "release", "fail",
];

pub(crate) fn status(document: &Document) -> Option<&str> {
    document
        .field("accord.status")
        .or_else(|| document.field("accordStatus"))
}

pub(crate) fn is_known_status(status: &str) -> bool {
    STATUSES.contains(&status) || LEGACY_STATUSES.contains(&status)
}

pub(crate) fn status_for_action(action: &str) -> Option<&'static str> {
    match action {
        "claim" => Some("claimed"),
        "deliver" => Some("delivered"),
        "rework" => Some("rework"),
        "block" => Some("blocked"),
        "resume" => Some("claimed"),
        "release" => Some("ready"),
        "fail" => Some("failed"),
        _ => None,
    }
}

pub(crate) fn event_name(action: &str) -> &'static str {
    match action {
        "claim" => "accord.claimed",
        "deliver" => "accord.delivered",
        "rework" => "accord.rework",
        "block" => "accord.blocked",
        "resume" => "accord.resumed",
        "release" => "accord.released",
        "fail" => "accord.failed",
        _ => "accord.updated",
    }
}

pub(crate) fn validate_transition(action: &str, previous_status: &str) -> Result<(), String> {
    match action {
        "rework" if previous_status != "delivered" && previous_status != "rework" => Err(
            format!("accord rework requires current accord.status=delivered; current status is {previous_status}"),
        ),
        "claim" | "deliver" | "block" | "fail" | "release" | "resume" if previous_status == "accepted" => Err(
            format!("accepted accord cannot transition with `tandem accord {action}`"),
        ),
        _ => Ok(()),
    }
}

/// Only the legacy-ready/claimed alignment is a visual suggestion. Delivered,
/// accepted, blocked, failed, and rework statuses do not imply a workflow state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StateEffect<'a> {
    pub(crate) state: Option<&'a str>,
}

/// Decide workflow effects for an accord action. The application layer only
/// applies this protocol result to the stored document. Review metadata does
/// not gate workflow transitions in protocol 0.3.0: reworking a Task that is
/// in validation returns it to in-progress directly.
pub(crate) fn state_effect<'a>(action: &str, current_state: &'a str) -> StateEffect<'a> {
    if action == "claim" && current_state == "todo" {
        return StateEffect {
            state: Some("in-progress"),
        };
    }
    if action == "rework" && current_state == "validation" {
        return StateEffect {
            state: Some("in-progress"),
        };
    }
    StateEffect { state: None }
}

pub(crate) fn state_sync_target<'a>(status: &str, current_state: &'a str) -> Option<&'a str> {
    (status == "claimed" && current_state == "todo").then_some("in-progress")
}

pub(crate) fn state_divergence_warning(document: &Document) -> Option<String> {
    let status = status(document)?;
    let state = document.field("state")?;
    let expected = state_sync_target(status, state)?;
    Some(format!(
        "{} has workflow state `{state}` but accord.status `{status}` suggests `{expected}`; preserving recorded state until a mutation synchronizes it.",
        document.id()
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn state_effects_cover_all_accord_actions_and_rework() {
        for action in ["deliver", "accept", "block", "fail", "release", "resume"] {
            assert_eq!(
                state_effect(action, "in-progress"),
                StateEffect { state: None }
            );
        }
        assert_eq!(
            state_effect("claim", "todo"),
            StateEffect {
                state: Some("in-progress")
            }
        );
        assert_eq!(state_effect("claim", "in-progress").state, None);
        assert_eq!(
            state_effect("rework", "validation"),
            StateEffect {
                state: Some("in-progress")
            }
        );
        assert_eq!(state_effect("rework", "in-progress").state, None);
    }

    #[test]
    fn protocol_0_3_transition_matrix_is_explicit() {
        assert_eq!(status_for_action("claim"), Some("claimed"));
        assert_eq!(status_for_action("deliver"), Some("delivered"));
        assert_eq!(status_for_action("rework"), Some("rework"));
        assert_eq!(status_for_action("block"), Some("blocked"));
        assert_eq!(status_for_action("resume"), Some("claimed"));
        assert_eq!(status_for_action("release"), Some("ready"));
        assert_eq!(status_for_action("fail"), Some("failed"));
        assert!(validate_transition("rework", "delivered").is_ok());
        assert!(validate_transition("resume", "blocked").is_ok());
        assert!(validate_transition("release", "claimed").is_ok());
        assert!(validate_transition("fail", "claimed").is_ok());
        assert!(validate_transition("deliver", "accepted").is_err());
    }

    #[test]
    fn ready_requires_acceptance_at_the_application_boundary() {
        assert!(STATUSES.contains(&"ready"));
        assert!(!ACTIONS.contains(&"accept"));
    }

    #[test]
    fn workflow_alignment_is_visible_without_collapsing_state() {
        let document = Document::new(
            HashMap::from([
                ("id".to_string(), "task-1".to_string()),
                ("state".to_string(), "in-progress".to_string()),
                ("accord.status".to_string(), "delivered".to_string()),
            ]),
            String::new(),
        );
        assert!(state_divergence_warning(&document).is_none());
        assert_eq!(document.field("state"), Some("in-progress"));
    }
}
