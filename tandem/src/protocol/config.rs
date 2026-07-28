//! Logical project configuration values.
//!
//! Workflow-state semantics live in [`super::workflow`]. See the normative
//! [workspace config fields](../../../protocol/plan/spec.md#workspace-config-fields).

pub(crate) const PROTOCOL_VERSION: &str = "0.2.0";
pub(crate) const LEGACY_PROTOCOL_VERSION: &str = "0.1.0";

pub(crate) const RULE_CATEGORIES: [&str; 4] = ["always", "never", "prefer", "context"];
pub(crate) const DECISION_STATUSES: &[&str] = &[
    "proposed",
    "accepted",
    "rejected",
    "deprecated",
    "superseded",
    "withdrawn",
];

#[derive(Debug, Clone)]
pub(crate) struct RuleItem {
    pub(crate) id: usize,
    pub(crate) rule: String,
    pub(crate) source: Option<String>,
}

pub(crate) type RulesByCategory = std::collections::BTreeMap<String, Vec<RuleItem>>;
