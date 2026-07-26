//! Logical Tandem document values and fixed document vocabulary.
//!
//! See the normative [document field reference](../../../protocol/plan/spec.md#v0-field-reference).

use std::collections::HashMap;

use yaml_rust2::{Yaml, YamlLoader};

pub(crate) const PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];
pub(crate) const EFFORTS: &[&str] = &["trivial", "small", "medium", "large"];
pub(crate) const TASK_KINDS: &[&str] = &["epic"];
pub(crate) const SUPPORTED_DOCUMENT_TYPES: &[&str] = &["task", "decision"];

/// Parsed document meaning. Project code retains its source path and location
/// separately so this value remains independent of concrete filesystem access.
#[derive(Debug, Clone)]
pub(crate) struct Document {
    pub(crate) fields: HashMap<String, String>,
    pub(crate) body: String,
}

impl Document {
    pub(crate) fn new(mut fields: HashMap<String, String>, body: String) -> Self {
        normalize_fields(&mut fields);
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

/// Canonical compatibility normalization for legacy flat metadata aliases.
/// Project parsing retains representation; protocol defines these field meanings.
pub(crate) fn normalize_fields(fields: &mut HashMap<String, String>) {
    copy_first_alias(fields, "accordStatus", &["accord.status"]);
    copy_first_alias(fields, "reviewStatus", &["review.status"]);
    copy_first_alias(fields, "completionSummary", &["completion.summary"]);
    copy_first_alias(
        fields,
        "completionValidation",
        &[
            "completion.validation",
            "completion.validation.summary",
            "completion.validation.status",
        ],
    );
    copy_first_alias(fields, "completionReviewer", &["completion.reviewer"]);
    copy_first_alias(fields, "filesChanged", &["completion.filesChanged"]);
}

fn copy_first_alias(fields: &mut HashMap<String, String>, alias: &str, sources: &[&str]) {
    if fields.contains_key(alias) {
        return;
    }
    for source in sources {
        if let Some(value) = fields.get(*source).cloned() {
            fields.insert(alias.to_string(), value);
            return;
        }
    }
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
    fn preserves_document_values_without_interpreting_lifecycle_metadata() {
        let document = Document::new(
            HashMap::from([
                ("completionSummary".to_string(), "Done".to_string()),
                ("custom.lifecycle".to_string(), "passed".to_string()),
            ]),
            String::new(),
        );

        assert_eq!(document.field("completionSummary"), Some("Done"));
        assert_eq!(document.field("custom.lifecycle"), Some("passed"));
    }
}
