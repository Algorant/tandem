//! Logical project configuration values.
//!
//! Workflow-state semantics live in [`super::workflow`]. See the normative
//! [workspace config fields](../../../protocol/plan/spec.md#workspace-config-fields).

pub(crate) const PROTOCOL_VERSION: &str = "0.2.0";

pub(crate) fn default_project_config(title: &str) -> String {
    let quoted_title = yaml_double_quote(title);
    format!(
        "---\nprotocolVersion: {PROTOCOL_VERSION}\ntype: workspace\ntitle: {quoted_title}\nstates:\n  - id: todo\n    title: To Do\n  - id: in-progress\n    title: In Progress\n  - id: validation\n    title: Validation\nrules:\n  always: []\n  never: []\n  prefer: []\n  context: []\n---\n\n# {title}\n"
    )
}

fn yaml_double_quote(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}
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
