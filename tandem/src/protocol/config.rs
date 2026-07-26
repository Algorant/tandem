//! Logical workspace configuration values and workflow-state semantics.
//!
//! See the normative [workspace config fields](../../../protocol/plan/spec.md#workspace-config-fields).

use yaml_rust2::Yaml;

pub(crate) const PROTOCOL_VERSION: &str = "0.2.0";
pub(crate) const LEGACY_PROTOCOL_VERSION: &str = "0.1.0";
pub(crate) const DEFAULT_STATES: &[&str] = &["todo", "in-progress", "validation"];
pub(crate) const LEGACY_REVIEW_STATE: &str = "review";
pub(crate) const VALIDATION_STATE: &str = "validation";

pub(crate) fn workflow_states(root: Option<&Yaml>) -> Vec<String> {
    let mut states = Vec::new();
    if let Some(states_yaml) = root.and_then(|root| yaml_mapping_value(root, "states")) {
        match states_yaml {
            Yaml::Array(items) => {
                for item in items {
                    if let Some(state) = yaml_scalar_to_string(item)
                        .or_else(|| yaml_mapping_value(item, "id").and_then(yaml_scalar_to_string))
                    {
                        if !state.trim().is_empty() {
                            states.push(state);
                        }
                    }
                }
            }
            _ => {
                if let Some(state) = yaml_scalar_to_string(states_yaml) {
                    if !state.trim().is_empty() {
                        states.push(state);
                    }
                }
            }
        }
    }
    if states.is_empty() {
        states.extend(DEFAULT_STATES.iter().map(|state| (*state).to_string()));
    }
    states
}

pub(crate) fn state_matches_filter(actual: Option<&str>, requested: &str) -> bool {
    actual == Some(requested)
        || (requested == VALIDATION_STATE && actual == Some(LEGACY_REVIEW_STATE))
        || (requested == LEGACY_REVIEW_STATE && actual == Some(VALIDATION_STATE))
}

pub(crate) fn is_known_or_legacy_state(states: &[String], state: &str) -> bool {
    states.iter().any(|known| known == state)
        || (state == LEGACY_REVIEW_STATE && states.iter().any(|known| known == VALIDATION_STATE))
        || (state == VALIDATION_STATE && states.iter().any(|known| known == LEGACY_REVIEW_STATE))
}

pub(crate) fn display_known_states(states: &[String]) -> String {
    let mut display = states.to_vec();
    if states.iter().any(|state| state == VALIDATION_STATE)
        && !states.iter().any(|state| state == LEGACY_REVIEW_STATE)
    {
        display.push(format!("{LEGACY_REVIEW_STATE} (legacy alias)"));
    } else if states.iter().any(|state| state == LEGACY_REVIEW_STATE)
        && !states.iter().any(|state| state == VALIDATION_STATE)
    {
        display.push(format!("{VALIDATION_STATE} (preferred alias)"));
    }
    display.join(", ")
}

fn yaml_scalar_to_string(value: &Yaml) -> Option<String> {
    match value {
        Yaml::String(value) | Yaml::Real(value) => Some(value.clone()),
        Yaml::Integer(value) => Some(value.to_string()),
        Yaml::Boolean(value) => Some(value.to_string()),
        Yaml::Null | Yaml::BadValue | Yaml::Array(_) | Yaml::Hash(_) | Yaml::Alias(_) => None,
    }
}

fn yaml_mapping_value<'a>(root: &'a Yaml, key: &str) -> Option<&'a Yaml> {
    root.as_hash()?.iter().find_map(|(candidate, value)| {
        (yaml_scalar_to_string(candidate).as_deref() == Some(key)).then_some(value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_states_use_configured_values_or_canonical_defaults() {
        let configured = Yaml::Array(vec![Yaml::String("queued".to_string())]);
        let root = Yaml::Hash(
            [(Yaml::String("states".to_string()), configured)]
                .into_iter()
                .collect(),
        );

        assert_eq!(workflow_states(Some(&root)), ["queued"]);
        assert_eq!(workflow_states(None), ["todo", "in-progress", "validation"]);
    }

    #[test]
    fn validation_and_review_remain_legacy_aliases() {
        let states = vec!["validation".to_string()];
        assert!(is_known_or_legacy_state(&states, "review"));
        assert!(state_matches_filter(Some("review"), "validation"));
        assert_eq!(
            display_known_states(&states),
            "validation, review (legacy alias)"
        );
    }
}
