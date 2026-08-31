//! One-file-per-rule project store for Rule records (protocol 0.3.0, D46/D49).
//!
//! Rules live as exactly one Markdown file per rule under `.tandem/rules/`
//! with composite ids like `always-12`, the category stored in the record,
//! optional `source`, timestamps, and the rule text as the Markdown body.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{display_path, split_frontmatter, write_atomic, yaml_double_quote};
use crate::protocol::config::{RuleItem, RulesByCategory, RULE_CATEGORIES};
use crate::CliError;

pub(crate) fn empty_rules() -> RulesByCategory {
    let mut rules = BTreeMap::new();
    for category in RULE_CATEGORIES {
        rules.insert(category.to_string(), Vec::new());
    }
    rules
}

/// Parses a composite rule id like `always-12` into (category, number).
pub(crate) fn parse_rule_id(id: &str) -> Option<(String, usize)> {
    for category in RULE_CATEGORIES {
        if let Some(number) = id
            .strip_prefix(&format!("{category}-"))
            .and_then(|number| number.parse::<usize>().ok())
        {
            if number > 0 {
                return Some((category.to_string(), number));
            }
        }
    }
    None
}

/// Loads all Rule files grouped into the category display map.
pub(crate) fn rules_by_category(dir: &Path) -> Result<RulesByCategory, CliError> {
    let mut rules = empty_rules();
    for rule in read_rule_files(dir)? {
        rules
            .entry(rule.category.clone())
            .or_default()
            .push(RuleItem {
                id: rule
                    .id
                    .rsplit_once('-')
                    .and_then(|(_, number)| number.parse::<usize>().ok())
                    .unwrap_or(0),
                rule: rule.text,
                source: rule.source,
            });
    }
    Ok(rules)
}

/// Deletes one Rule file by composite id.
pub(crate) fn delete_rule_file(dir: &Path, id: &str) -> Result<(), CliError> {
    if parse_rule_id(id).is_none() {
        return Err(CliError::user(format!(
            "invalid rule id `{id}`; expected <category>-<number> like always-12"
        )));
    }
    let path = dir.join(format!("{id}.md"));
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(CliError::user(format!("rule not found: {id}")))
        }
        Err(error) => Err(CliError::user(format!(
            "failed to delete {}: {error}",
            display_path(&path)
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuleRecord {
    pub(crate) id: String,
    pub(crate) category: String,
    pub(crate) source: Option<String>,
    pub(crate) created_at: Option<String>,
    pub(crate) updated_at: Option<String>,
    pub(crate) text: String,
    pub(crate) path: PathBuf,
}

pub(crate) fn read_rule_files(dir: &Path) -> Result<Vec<RuleRecord>, CliError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(dir)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| path.extension().and_then(|e| e.to_str()) == Some("md"));
    paths.sort();
    paths
        .into_iter()
        .map(|path| read_rule_file(&path))
        .collect()
}

pub(crate) fn read_rule_file(path: &Path) -> Result<RuleRecord, CliError> {
    let content = fs::read_to_string(path)
        .map_err(|e| CliError::user(format!("failed to read {}: {e}", display_path(path))))?;
    let (frontmatter, text) = split_frontmatter(&content)
        .map_err(|e| CliError::user(format!("Parse failure: {}: {e}", display_path(path))))?;
    let fields = super::parse_frontmatter_fields(&frontmatter).map_err(CliError::user)?;
    let id = fields
        .get("id")
        .cloned()
        .ok_or_else(|| CliError::user(format!("Rule {} is missing id", display_path(path))))?;
    let category = fields
        .get("category")
        .cloned()
        .ok_or_else(|| CliError::user(format!("Rule {id} is missing category")))?;
    validate_rule_category(&category)?;
    if !rule_id_matches(&id, &category) {
        return Err(CliError::user(format!(
            "Rule {id} does not match category {category}"
        )));
    }
    if text.trim().is_empty() {
        return Err(CliError::user(format!("Rule {id} has empty text")));
    }
    Ok(RuleRecord {
        id,
        category,
        source: fields.get("source").cloned(),
        created_at: fields.get("createdAt").cloned(),
        updated_at: fields.get("updatedAt").cloned(),
        text,
        path: path.to_path_buf(),
    })
}

pub(crate) fn next_rule_id(dir: &Path, category: &str) -> Result<String, CliError> {
    validate_rule_category(category)?;
    let prefix = format!("{category}-");
    let mut next = 1usize;
    for rule in read_rule_files(dir)? {
        if let Some(number) = rule
            .id
            .strip_prefix(&prefix)
            .and_then(|n| n.parse::<usize>().ok())
        {
            next = next.max(number.saturating_add(1));
        }
    }
    Ok(format!("{prefix}{next}"))
}

pub(crate) fn write_rule_file(dir: &Path, rule: &RuleRecord) -> Result<(), CliError> {
    validate_rule_category(&rule.category)?;
    if !rule_id_matches(&rule.id, &rule.category) {
        return Err(CliError::user(format!(
            "Rule {} does not match category {}",
            rule.id, rule.category
        )));
    }
    if rule.text.trim().is_empty() {
        return Err(CliError::user("rule text must not be empty"));
    }
    fs::create_dir_all(dir)?;
    let path = dir.join(format!("{}.md", rule.id));
    let mut frontmatter = format!("id: {}\ncategory: {}\n", rule.id, rule.category);
    if let Some(source) = &rule.source {
        frontmatter.push_str(&format!("source: {}\n", yaml_double_quote(source)));
    }
    if let Some(created) = &rule.created_at {
        frontmatter.push_str(&format!("createdAt: {}\n", yaml_double_quote(created)));
    }
    if let Some(updated) = &rule.updated_at {
        frontmatter.push_str(&format!("updatedAt: {}\n", yaml_double_quote(updated)));
    }
    write_atomic(&path, &format!("---\n{frontmatter}---\n{}\n", rule.text))
}

fn validate_rule_category(category: &str) -> Result<(), CliError> {
    if RULE_CATEGORIES.contains(&category) {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "invalid Rule category `{category}`; expected always, never, prefer, or context"
        )))
    }
}
fn rule_id_matches(id: &str, category: &str) -> bool {
    id.strip_prefix(&format!("{category}-"))
        .is_some_and(|n| n.parse::<usize>().is_ok_and(|n| n > 0))
}
