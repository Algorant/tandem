//! Canonical accord vocabulary, transitions, and workflow alignment.

use super::document::{parse_field_values, Document};
use super::workflow::{LEGACY_REVIEW_STATE, VALIDATION_STATE};

#[derive(Debug, Clone, Default)]
pub(crate) struct AccordRecord {
    pub(crate) status: String,
    pub(crate) assignee: Option<String>,
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
            assignee: doc.field("accord.assignee").map(str::to_string),
            claimed_at: doc.field("accord.claimedAt").map(str::to_string),
            delivered_at: doc.field("accord.deliveredAt").map(str::to_string),
            deliverables: doc
                .field("accord.deliverables")
                .map(parse_field_values)
                .unwrap_or_default(),
            validations: doc
                .field("accord.validation.commands")
                .or_else(|| doc.field("accord.validation"))
                .or_else(|| doc.field("accord.validations"))
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
    "claimed",
    "delivered",
    "accepted",
    "rework",
    "failed",
    "blocked",
];
pub(crate) const LEGACY_STATUSES: &[&str] = &["ready"];
pub(crate) const ACTIONS: &[&str] = &["claim", "deliver", "accept", "rework", "block", "fail"];

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
        "accept" => Some("accepted"),
        "rework" => Some("rework"),
        "block" => Some("blocked"),
        "fail" => Some("failed"),
        _ => None,
    }
}

pub(crate) fn event_name(action: &str) -> &'static str {
    match action {
        "claim" => "accord.claimed",
        "deliver" => "accord.delivered",
        "accept" => "accord.accepted",
        "rework" => "accord.rework",
        "block" => "accord.blocked",
        "fail" => "accord.failed",
        _ => "accord.updated",
    }
}

pub(crate) fn validate_transition(action: &str, previous_status: &str) -> Result<(), String> {
    match action {
        "accept" if previous_status != "delivered" && previous_status != "accepted" => Err(
            format!("accord accept requires current accord.status=delivered; current status is {previous_status}"),
        ),
        "rework" if previous_status != "delivered" && previous_status != "rework" => Err(
            format!("accord rework requires current accord.status=delivered; current status is {previous_status}"),
        ),
        "claim" | "deliver" | "block" | "fail" if previous_status == "accepted" => Err(
            format!("accepted accord cannot transition with `tandem accord {action}`"),
        ),
        _ => Ok(()),
    }
}

pub(crate) fn state_sync_target<'a>(status: &str, current_state: &'a str) -> Option<&'a str> {
    match status {
        "claimed" if current_state == "todo" => Some("in-progress"),
        "delivered" | "accepted"
            if matches!(current_state, "todo" | "in-progress" | LEGACY_REVIEW_STATE) =>
        {
            Some(VALIDATION_STATE)
        }
        "rework" if matches!(current_state, VALIDATION_STATE | LEGACY_REVIEW_STATE) => {
            Some("in-progress")
        }
        _ => None,
    }
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
    fn workflow_alignment_is_visible_without_collapsing_state() {
        let document = Document::new(
            HashMap::from([
                ("id".to_string(), "task-1".to_string()),
                ("state".to_string(), "in-progress".to_string()),
                ("accord.status".to_string(), "delivered".to_string()),
            ]),
            String::new(),
        );
        assert!(state_divergence_warning(&document)
            .unwrap()
            .contains("suggests `validation`"));
        assert_eq!(document.field("state"), Some("in-progress"));
    }
}
