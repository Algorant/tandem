//! Canonical workflow and completion semantics.
//!
//! Workflow `state` is configurable project data. Completion archives a task
//! into Logs; it is not another workflow state.

use yaml_rust2::Yaml;

use super::document::{parse_field_values, Document};

pub(crate) const DEFAULT_STATES: &[&str] = &["todo", "in-progress", "validation"];
pub(crate) const LEGACY_REVIEW_STATE: &str = "review";
pub(crate) const VALIDATION_STATE: &str = "validation";
pub(crate) const RESOLUTION_OUTCOME_COMPLETED: &str = "completed";
pub(crate) const RESOLUTION_OUTCOME_CANCELED: &str = "canceled";
pub(crate) const RESOLUTION_OUTCOME_FAILED: &str = "failed";

#[derive(Debug, Clone, Default)]
pub(crate) struct ResolutionRecord {
    pub(crate) outcome: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) reviewer: Option<String>,
}
pub(crate) const RESOLUTION_OUTCOMES: &[&str] = &[
    RESOLUTION_OUTCOME_COMPLETED,
    RESOLUTION_OUTCOME_CANCELED,
    RESOLUTION_OUTCOME_FAILED,
];

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

/// Resolution metadata on archived Logs (minimal D47 record). Falls back to
/// legacy completion fields so imported history stays readable.
pub(crate) fn resolution_note(document: &Document) -> Option<&str> {
    document
        .field("resolution.note")
        .or_else(|| document.field("completion.summary"))
        .or_else(|| document.field("completionSummary"))
}

pub(crate) fn resolution_outcome(document: &Document) -> &str {
    document
        .field("resolution.outcome")
        .or_else(|| document.field("completion.outcome"))
        .unwrap_or(RESOLUTION_OUTCOME_COMPLETED)
}

pub(crate) fn resolution_reviewer(document: &Document) -> Option<&str> {
    document
        .field("resolution.reviewer")
        .or_else(|| document.field("completion.reviewer"))
        .or_else(|| document.field("completionReviewer"))
}

pub(crate) fn resolution_files_changed(document: &Document) -> Vec<String> {
    document
        .field("resolution.filesChanged")
        .or_else(|| document.field("completion.filesChanged"))
        .or_else(|| document.field("filesChanged"))
        .map(parse_field_values)
        .unwrap_or_default()
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
    use std::collections::HashMap;

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
    fn resolution_reads_nested_and_legacy_values() {
        let document = Document::new(
            HashMap::from([
                ("resolution.note".to_string(), "Done".to_string()),
                ("resolution.reviewer".to_string(), "owner".to_string()),
                (
                    "resolution.filesChanged".to_string(),
                    "[src/main.rs]".to_string(),
                ),
            ]),
            String::new(),
        );
        assert_eq!(resolution_note(&document), Some("Done"));
        assert_eq!(resolution_outcome(&document), "completed");
        assert_eq!(resolution_reviewer(&document), Some("owner"));
        assert_eq!(resolution_files_changed(&document), ["src/main.rs"]);

        let legacy = Document::new(
            HashMap::from([
                ("completion.summary".to_string(), "Done".to_string()),
                ("completion.outcome".to_string(), "canceled".to_string()),
            ]),
            String::new(),
        );
        assert_eq!(resolution_note(&legacy), Some("Done"));
        assert_eq!(resolution_outcome(&legacy), "canceled");
    }
}
