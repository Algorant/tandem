//! Shared project Rules mutation operations.

use crate::app::support::{append_event, document_exists};
use crate::project::rules::{parse_rules_from_content, patch_rules_category_content};
use crate::project::write::{ensure_file_unchanged, read_file_snapshot};
use crate::project::{write_atomic, TandemProject};
use crate::protocol::config::{RuleItem, RULE_CATEGORIES};
use crate::CliError;

#[derive(Debug)]
pub(crate) struct MutationOutcome {
    pub(crate) category: String,
    pub(crate) id: usize,
    pub(crate) rule: String,
    pub(crate) warning: Option<String>,
}

#[derive(Debug)]
pub(crate) struct DeleteOutcome {
    pub(crate) category: String,
    pub(crate) id: usize,
}

pub(crate) fn add(
    project: &TandemProject,
    category: &str,
    rule: &str,
    source: Option<String>,
) -> Result<MutationOutcome, CliError> {
    validate_rule_category(category)?;
    let rule = require_rule_text(rule, "rules add requires --rule <text>")?;
    let source = normalized_source(source);
    let warning = missing_source_warning(project, source.as_deref())?;
    let (content, signature) = read_file_snapshot(&project.config_path)?;
    let mut rules = parse_rules_from_content(&content, &project.config_path)?;
    let id = rules
        .get(category)
        .into_iter()
        .flatten()
        .map(|item| item.id)
        .max()
        .unwrap_or(0)
        + 1;
    rules
        .entry(category.to_string())
        .or_default()
        .push(RuleItem {
            id,
            rule: rule.to_string(),
            source,
        });
    let patched = patch_rules_category_content(&content, category, &rules)?;
    ensure_file_unchanged(&project.config_path, &signature)?;
    write_atomic(&project.config_path, &patched)?;
    append_event(
        project,
        "rules.updated",
        "rules",
        &format!("Added rule {id} to {category}"),
    )?;
    Ok(MutationOutcome {
        category: category.to_string(),
        id,
        rule: rule.to_string(),
        warning,
    })
}

pub(crate) fn edit(
    project: &TandemProject,
    category: &str,
    id: usize,
    rule: &str,
    source: Option<String>,
) -> Result<MutationOutcome, CliError> {
    validate_rule_category(category)?;
    let rule = require_rule_text(rule, "rules edit requires --rule <text>")?;
    let source = source.map(|value| normalized_source(Some(value)));
    let warning = missing_source_warning(project, source.as_ref().and_then(Option::as_deref))?;
    let (content, signature) = read_file_snapshot(&project.config_path)?;
    let mut rules = parse_rules_from_content(&content, &project.config_path)?;
    let item = rules
        .entry(category.to_string())
        .or_default()
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| CliError::user(format!("rule not found: {category} #{id}")))?;
    item.rule = rule.to_string();
    if let Some(source) = source {
        item.source = source;
    }
    let patched = patch_rules_category_content(&content, category, &rules)?;
    ensure_file_unchanged(&project.config_path, &signature)?;
    write_atomic(&project.config_path, &patched)?;
    append_event(
        project,
        "rules.updated",
        "rules",
        &format!("Edited rule {id} in {category}"),
    )?;
    Ok(MutationOutcome {
        category: category.to_string(),
        id,
        rule: rule.to_string(),
        warning,
    })
}

pub(crate) fn delete(
    project: &TandemProject,
    category: &str,
    id: usize,
) -> Result<DeleteOutcome, CliError> {
    validate_rule_category(category)?;
    let (content, signature) = read_file_snapshot(&project.config_path)?;
    let mut rules = parse_rules_from_content(&content, &project.config_path)?;
    let items = rules.entry(category.to_string()).or_default();
    let before = items.len();
    items.retain(|item| item.id != id);
    if items.len() == before {
        return Err(CliError::user(format!("rule not found: {category} #{id}")));
    }
    let patched = patch_rules_category_content(&content, category, &rules)?;
    ensure_file_unchanged(&project.config_path, &signature)?;
    write_atomic(&project.config_path, &patched)?;
    append_event(
        project,
        "rules.updated",
        "rules",
        &format!("Deleted rule {id} from {category}"),
    )?;
    Ok(DeleteOutcome {
        category: category.to_string(),
        id,
    })
}

pub(crate) fn validate_rule_category(category: &str) -> Result<(), CliError> {
    if RULE_CATEGORIES.contains(&category) {
        Ok(())
    } else {
        Err(CliError::usage(format!(
            "unknown rule category `{category}`; use always, never, prefer, or context"
        )))
    }
}

fn require_rule_text<'a>(value: &'a str, message: &str) -> Result<&'a str, CliError> {
    let value = value.trim();
    if value.is_empty() {
        Err(CliError::usage(message))
    } else {
        Ok(value)
    }
}

fn normalized_source(source: Option<String>) -> Option<String> {
    source.and_then(|value| (!value.trim().is_empty()).then_some(value))
}

fn missing_source_warning(
    project: &TandemProject,
    source: Option<&str>,
) -> Result<Option<String>, CliError> {
    if let Some(source) = source {
        if !document_exists(project, source)? {
            return Ok(Some(format!("rule source not found: {source}")));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn add_edit_delete_preserve_config_and_return_source_warning() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-rules-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.2.0\nstates: [todo, in-progress, validation]\nunknown: retain\n---\nbody\n",
        )
        .unwrap();
        let added = add(&project, "always", "Keep it", Some(" missing ".to_string())).unwrap();
        assert_eq!(added.id, 1);
        assert_eq!(
            added.warning.as_deref(),
            Some("rule source not found:  missing ")
        );
        edit(&project, "always", 1, "Keep all", Some(String::new())).unwrap();
        let edited = fs::read_to_string(&project.config_path).unwrap();
        assert!(edited.contains("unknown: retain"));
        assert!(edited.ends_with("body\n"));
        assert!(edited.contains("Keep all"));
        assert!(!edited.contains("source:"));
        delete(&project, "always", 1).unwrap();
        assert!(!fs::read_to_string(&project.config_path)
            .unwrap()
            .contains("Keep all"));
        fs::remove_dir_all(root).unwrap();
    }
}
