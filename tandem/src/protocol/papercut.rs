//! Protocol meaning for lightweight Papercut inbox records.
//!
//! Papercuts are not general Tandem documents. They have their own identity,
//! status, and validation contract and never enter hierarchy or workflow code.

use std::collections::HashMap;

use crate::protocol::document::parse_field_values;

pub(crate) const STATUSES: &[&str] = &["open", "resolved"];

#[derive(Debug, Clone)]
pub(crate) struct Papercut {
    pub(crate) fields: HashMap<String, String>,
    pub(crate) body: String,
}

impl Papercut {
    pub(crate) fn new(fields: HashMap<String, String>, body: String) -> Self {
        Self { fields, body }
    }

    pub(crate) fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    pub(crate) fn id(&self) -> &str {
        self.field("id").unwrap_or("")
    }

    pub(crate) fn title(&self) -> &str {
        self.field("title").unwrap_or("")
    }

    pub(crate) fn status(&self) -> &str {
        self.field("status").unwrap_or("")
    }

    pub(crate) fn values(&self, key: &str) -> Vec<String> {
        self.field(key).map(parse_field_values).unwrap_or_default()
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        for key in ["id", "title", "status", "createdAt", "updatedAt"] {
            if self.field(key).is_none_or(|value| value.trim().is_empty()) {
                errors.push(format!("missing required field `{key}`"));
            }
        }
        if !self.id().is_empty() && papercut_number(self.id()).is_none() {
            errors.push(format!(
                "invalid Papercut ID `{}`; expected `papercut-N`",
                self.id()
            ));
        }
        if !self.status().is_empty() && !STATUSES.contains(&self.status()) {
            errors.push(format!(
                "invalid Papercut status `{}`; expected one of: {}",
                self.status(),
                STATUSES.join(", ")
            ));
        }
        match self.status() {
            "resolved" => {
                for key in ["resolution.note", "resolution.resolvedAt"] {
                    if self.field(key).is_none_or(|value| value.trim().is_empty()) {
                        errors.push(format!("resolved Papercut is missing `{key}`"));
                    }
                }
            }
            "open"
                if self
                    .fields
                    .keys()
                    .any(|key| key == "resolution" || key.starts_with("resolution.")) =>
            {
                errors.push("open Papercut must not contain resolution metadata".to_string());
            }
            _ => {}
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

pub(crate) fn papercut_number(id: &str) -> Option<usize> {
    let value = id.strip_prefix("papercut-")?;
    if value.is_empty() || value.starts_with('0') {
        return None;
    }
    value.parse::<usize>().ok().filter(|number| *number > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(values: &[(&str, &str)]) -> HashMap<String, String> {
        values
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    #[test]
    fn validates_open_and_resolved_records_without_document_taxonomy() {
        let open = Papercut::new(
            fields(&[
                ("id", "papercut-1"),
                ("title", "Friction"),
                ("status", "open"),
                ("createdAt", "now"),
                ("updatedAt", "now"),
                ("unknown", "keep"),
            ]),
            "Body".to_string(),
        );
        assert!(open.validate().is_ok());
        assert_eq!(open.field("unknown"), Some("keep"));

        let malformed = Papercut::new(
            fields(&[
                ("id", "task-1"),
                ("title", "Bad"),
                ("status", "resolved"),
                ("createdAt", "now"),
                ("updatedAt", "now"),
            ]),
            String::new(),
        );
        let error = malformed.validate().unwrap_err();
        assert!(error.contains("expected `papercut-N`"));
        assert!(error.contains("resolution.note"));
    }
}
