//! Canonical event names and minimal audit-envelope shape.
//!
//! Project code owns actor-log discovery, sequence allocation, and JSONL
//! append. This module only defines what an event means.

/// Required v0.2 per-actor audit fields. Project code supplies the actor and
/// per-actor sequence after resolving its concrete event log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalEventEnvelope<'a> {
    pub(crate) ts: &'a str,
    pub(crate) event: &'a str,
    pub(crate) id: &'a str,
    pub(crate) summary: &'a str,
    pub(crate) actor: &'a str,
    pub(crate) seq: u64,
}

impl CanonicalEventEnvelope<'_> {
    pub(crate) fn required_fields() -> [&'static str; 6] {
        ["ts", "event", "id", "summary", "actor", "seq"]
    }
}

pub(crate) const TASK_CREATED: &str = "task.created";
pub(crate) const TASK_MOVED: &str = "task.moved";
pub(crate) const TASK_UPDATED: &str = "task.updated";
pub(crate) const TASK_COMPLETED: &str = "task.completed";
pub(crate) const TASK_CANCELED: &str = "task.canceled";
pub(crate) const DECISION_CREATED: &str = "decision.created";
pub(crate) const RULES_UPDATED: &str = "rules.updated";

pub(crate) fn is_known_name(name: &str) -> bool {
    matches!(
        name,
        TASK_CREATED
            | TASK_MOVED
            | TASK_UPDATED
            | TASK_COMPLETED
            | TASK_CANCELED
            | DECISION_CREATED
            | RULES_UPDATED
            | "accord.claimed"
            | "accord.delivered"
            | "accord.accepted"
            | "accord.rework"
            | "accord.blocked"
            | "accord.failed"
            | "validation.accepted"
            | "validation.rework"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_actor_identity_requires_all_v02_envelope_fields() {
        let event = CanonicalEventEnvelope {
            ts: "2026-07-26T00:00:00Z",
            event: TASK_CREATED,
            id: "task-1",
            summary: "Created task-1",
            actor: "pi",
            seq: 4,
        };
        assert_eq!(
            CanonicalEventEnvelope::required_fields(),
            ["ts", "event", "id", "summary", "actor", "seq"]
        );
        assert_eq!(format!("{}:{}", event.actor, event.seq), "pi:4");
        assert!(event.ts.contains('T') && is_known_name(event.event));
        assert!(!event.id.is_empty() && !event.summary.is_empty());
    }
}
