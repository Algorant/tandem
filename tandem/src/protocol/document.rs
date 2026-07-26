//! Logical Tandem document values and fixed document vocabulary.
//!
//! See the normative [document field reference](../../../protocol/plan/spec.md#v0-field-reference).

use std::collections::HashMap;

use yaml_rust2::{Yaml, YamlLoader};

pub(crate) const PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];
pub(crate) const EFFORTS: &[&str] = &["trivial", "small", "medium", "large"];
pub(crate) const TASK_KINDS: &[&str] = &["epic"];
pub(crate) const SUPPORTED_DOCUMENT_TYPES: &[&str] = &["task", "decision"];
pub(crate) const COMPLETION_OUTCOME_COMPLETED: &str = "completed";
pub(crate) const COMPLETION_OUTCOME_CANCELED: &str = "canceled";

/// Parsed document meaning. Project code retains its source path and location
/// separately so this value remains independent of concrete filesystem access.
#[derive(Debug, Clone)]
pub(crate) struct Document {
    pub(crate) fields: HashMap<String, String>,
    pub(crate) body: String,
}

impl Document {
    pub(crate) fn new(fields: HashMap<String, String>, body: String) -> Self {
        Self { fields, body }
    }

    pub(crate) fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    pub(crate) fn id(&self) -> &str {
        self.field("id").unwrap_or("")
    }

    pub(crate) fn doc_type(&self) -> &str {
        self.field("type").unwrap_or("task")
    }

    pub(crate) fn is_first_class_type(&self) -> bool {
        is_supported_document_type(self.doc_type())
    }

    pub(crate) fn kind(&self) -> Option<&str> {
        self.field("kind")
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub(crate) fn title(&self) -> &str {
        self.field("title").unwrap_or("")
    }

    pub(crate) fn accord_status(&self) -> Option<&str> {
        self.field("accord.status")
            .or_else(|| self.field("accordStatus"))
    }

    pub(crate) fn review_status(&self) -> Option<&str> {
        self.field("review.status")
            .or_else(|| self.field("reviewStatus"))
    }

    pub(crate) fn completion_summary(&self) -> Option<&str> {
        self.field("completion.summary")
            .or_else(|| self.field("completionSummary"))
    }

    pub(crate) fn completion_outcome(&self) -> &str {
        self.field("completion.outcome")
            .unwrap_or(COMPLETION_OUTCOME_COMPLETED)
    }

    pub(crate) fn completion_validation(&self) -> Option<&str> {
        self.field("completion.validation")
            .or_else(|| self.field("completion.validation.summary"))
            .or_else(|| self.field("completion.validation.status"))
            .or_else(|| self.field("completionValidation"))
    }

    pub(crate) fn completion_reviewer(&self) -> Option<&str> {
        self.field("completion.reviewer")
            .or_else(|| self.field("completionReviewer"))
    }

    pub(crate) fn has_metadata(&self, prefix: &str) -> bool {
        let nested_prefix = format!("{prefix}.");
        self.fields
            .keys()
            .any(|key| key == prefix || key.starts_with(&nested_prefix))
    }

    pub(crate) fn values(&self, key: &str) -> Vec<String> {
        self.field(key).map(parse_field_values).unwrap_or_default()
    }
}

pub(crate) fn accord_status(document: &Document) -> Option<&str> {
    document.accord_status()
}

pub(crate) fn review_status(document: &Document) -> Option<&str> {
    document.review_status()
}

pub(crate) fn completion_summary(document: &Document) -> Option<&str> {
    document.completion_summary()
}

pub(crate) fn completion_outcome(document: &Document) -> &str {
    document.completion_outcome()
}

pub(crate) fn completion_validation(document: &Document) -> Option<&str> {
    document.completion_validation()
}

pub(crate) fn completion_reviewer(document: &Document) -> Option<&str> {
    document.completion_reviewer()
}

pub(crate) fn completion_files_changed(document: &Document) -> Vec<String> {
    document
        .field("completion.filesChanged")
        .or_else(|| document.field("filesChanged"))
        .map(parse_field_values)
        .unwrap_or_default()
}

pub(crate) fn has_metadata(document: &Document, prefix: &str) -> bool {
    document.has_metadata(prefix)
}

pub(crate) fn is_supported_document_type(doc_type: &str) -> bool {
    SUPPORTED_DOCUMENT_TYPES.contains(&doc_type)
}

pub(crate) fn validate_task_kind(kind: &str) -> Result<(), String> {
    let kind = kind.trim();
    if kind.is_empty() {
        return Err("kind must not be empty when present".to_string());
    }
    if !TASK_KINDS.contains(&kind) {
        return Err(format!(
            "invalid kind `{kind}`; expected one of: {}",
            TASK_KINDS.join(", ")
        ));
    }
    Ok(())
}

pub(crate) fn parse_field_values(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Vec::new();
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        if let Ok(docs) = YamlLoader::load_from_str(trimmed) {
            if let Some(Yaml::Array(values)) = docs.first() {
                return values
                    .iter()
                    .filter_map(yaml_scalar_to_string)
                    .filter(|item| !item.is_empty())
                    .collect();
            }
        }
        return trimmed[1..trimmed.len() - 1]
            .split(',')
            .map(|item| parse_scalar_value(item.trim()))
            .filter(|item| !item.is_empty())
            .collect();
    }
    vec![parse_scalar_value(trimmed)]
        .into_iter()
        .filter(|item| !item.is_empty())
        .collect()
}

fn yaml_scalar_to_string(value: &Yaml) -> Option<String> {
    match value {
        Yaml::String(value) | Yaml::Real(value) => Some(value.clone()),
        Yaml::Integer(value) => Some(value.to_string()),
        Yaml::Boolean(value) => Some(value.to_string()),
        Yaml::Null | Yaml::BadValue | Yaml::Array(_) | Yaml::Hash(_) | Yaml::Alias(_) => None,
    }
}

fn parse_scalar_value(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    let without_comment = if value.starts_with('"') || value.starts_with('\'') {
        value
    } else {
        value.split(" #").next().unwrap_or(value).trim_end()
    };

    if let Some(stripped) = without_comment
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return unescape_double_quoted(stripped);
    }

    if let Some(stripped) = without_comment
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    {
        return stripped.replace("''", "'");
    }

    without_comment.trim().to_string()
}

fn unescape_double_quoted(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => output.push('\n'),
                Some('r') => output.push('\r'),
                Some('t') => output.push('\t'),
                Some('"') => output.push('"'),
                Some('\\') => output.push('\\'),
                Some(other) => {
                    output.push('\\');
                    output.push(other);
                }
                None => output.push('\\'),
            }
        } else {
            output.push(ch);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unknown_fields_and_markdown_body() {
        let document = Document::new(
            HashMap::from([
                ("id".to_string(), "task-1".to_string()),
                ("type".to_string(), "task".to_string()),
                ("custom.nested".to_string(), "keep".to_string()),
            ]),
            "## Notes\n\nKeep this body.\n".to_string(),
        );

        assert_eq!(document.field("custom.nested"), Some("keep"));
        assert_eq!(document.body, "## Notes\n\nKeep this body.\n");
        assert!(document.is_first_class_type());
        assert!(is_supported_document_type("decision"));
        assert!(!is_supported_document_type("bug"));
    }

    #[test]
    fn reads_legacy_completion_aliases_without_losing_fixed_outcome_default() {
        let document = Document::new(
            HashMap::from([
                ("completionSummary".to_string(), "Done".to_string()),
                ("completionValidation".to_string(), "passed".to_string()),
            ]),
            String::new(),
        );

        assert_eq!(document.completion_summary(), Some("Done"));
        assert_eq!(document.completion_validation(), Some("passed"));
        assert_eq!(document.completion_outcome(), "completed");
    }
}
