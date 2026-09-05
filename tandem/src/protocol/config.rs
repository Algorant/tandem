//! Logical project configuration values.
//!
//! Workflow-state semantics live in [`super::workflow`]. See the normative
//! [workspace config fields](../../../protocol/plan/spec.md#workspace-config-fields).

pub(crate) const PROTOCOL_VERSION: &str = "0.3.0";

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
pub(crate) const RULE_CATEGORIES: [&str; 4] = ["always", "never", "prefer", "context"];

/// Returns a diagnostic for the pre-0.3 workspace configuration layout.
/// Rules in the workspace `tandem.md` are legacy data and are not active;
/// active rules must be stored as individual files under `.tandem/rules/`.
pub(crate) fn embedded_rules_warning(root: Option<&yaml_rust2::Yaml>) -> Option<String> {
    let rules = root.and_then(|root| {
        root.as_hash()?
            .iter()
            .find_map(|(key, value)| (key.as_str() == Some("rules")).then_some(value))
    })?;
    let populated = yaml_value_is_populated(rules);
    populated.then(|| {
        "Legacy embedded rules found in the `rules:` block in tandem.md; those rules are not active. Move them to individual files under `.tandem/rules/`.".to_string()
    })
}
#[cfg(test)]
mod tests {
    use super::embedded_rules_warning;
    use yaml_rust2::YamlLoader;

    #[test]
    fn embedded_rules_are_reported_but_empty_defaults_are_not() {
        let empty = YamlLoader::load_from_str("rules:\n  always: []\n  never: []\n").unwrap();
        assert!(embedded_rules_warning(empty.first()).is_none());

        let populated = YamlLoader::load_from_str(
            "rules:\n  always:\n    - id: always-1\n      rule: Keep it explicit\n",
        )
        .unwrap();
        let warning = embedded_rules_warning(populated.first()).unwrap();
        assert!(warning.contains("rules:` block in tandem.md"));
        assert!(warning.contains("not active"));
    }
}

fn yaml_value_is_populated(value: &yaml_rust2::Yaml) -> bool {
    match value {
        yaml_rust2::Yaml::Hash(entries) => entries.values().any(yaml_value_is_populated),
        yaml_rust2::Yaml::Array(values) => values.iter().any(yaml_value_is_populated),
        yaml_rust2::Yaml::String(value) | yaml_rust2::Yaml::Real(value) => !value.is_empty(),
        yaml_rust2::Yaml::Null => false,
        _ => true,
    }
}

pub(crate) const DECISION_STATUSES: &[&str] = &[
    "proposed",
    "accepted",
    "rejected",
    "deprecated",
    "superseded",
];

#[derive(Debug, Clone)]
pub(crate) struct RuleItem {
    pub(crate) id: usize,
    pub(crate) rule: String,
    pub(crate) source: Option<String>,
}

pub(crate) type RulesByCategory = std::collections::BTreeMap<String, Vec<RuleItem>>;
