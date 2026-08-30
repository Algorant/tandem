//! Concrete parsing and byte-preserving patching of project Rules configuration.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use yaml_rust2::Yaml;

use super::{
    display_path, frontmatter_line_key, is_top_level_frontmatter_boundary, parse_frontmatter_yaml,
    read_frontmatter_yaml_file, split_frontmatter, write_atomic, yaml_double_quote,
    yaml_mapping_value, yaml_scalar_to_string,
};
use crate::protocol::config::{RuleItem, RulesByCategory, RULE_CATEGORIES};
use crate::CliError;

pub(crate) fn empty_rules() -> RulesByCategory {
    let mut rules = BTreeMap::new();
    for category in RULE_CATEGORIES {
        rules.insert(category.to_string(), Vec::new());
    }
    rules
}

pub(crate) fn read_rules(config_path: &Path) -> Result<RulesByCategory, CliError> {
    let root = read_frontmatter_yaml_file(config_path)?;
    Ok(parse_rules_from_yaml(root.as_ref()))
}

pub(crate) fn parse_rules_from_content(
    content: &str,
    path: &Path,
) -> Result<RulesByCategory, CliError> {
    let (frontmatter, _) = split_frontmatter(content).map_err(|message| {
        CliError::user(format!("Parse failure: {}: {message}", display_path(path)))
    })?;
    let root = parse_frontmatter_yaml(&frontmatter).map_err(|message| {
        CliError::user(format!(
            "Parse failure: {} frontmatter YAML: {message}",
            display_path(path)
        ))
    })?;
    Ok(parse_rules_from_yaml(root.as_ref()))
}

pub(crate) fn parse_rules_from_yaml(root: Option<&Yaml>) -> RulesByCategory {
    let mut rules = empty_rules();
    let Some(rules_yaml) = root.and_then(|root| yaml_mapping_value(root, "rules")) else {
        return rules;
    };
    for category in RULE_CATEGORIES {
        let Some(category_yaml) = yaml_mapping_value(rules_yaml, category) else {
            continue;
        };
        rules.insert(
            category.to_string(),
            parse_rule_category_items(category_yaml),
        );
    }
    rules
}

fn parse_rule_category_items(value: &Yaml) -> Vec<RuleItem> {
    match value {
        Yaml::Array(items) => items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| parse_rule_item(item, index + 1))
            .collect(),
        _ => parse_rule_item(value, 1).into_iter().collect(),
    }
}

fn parse_rule_item(value: &Yaml, fallback_id: usize) -> Option<RuleItem> {
    match value {
        Yaml::Hash(_) => {
            let id = yaml_mapping_value(value, "id")
                .and_then(yaml_scalar_to_string)
                .and_then(|value| value.parse().ok())
                .unwrap_or(fallback_id);
            let rule = yaml_mapping_value(value, "rule")
                .and_then(yaml_scalar_to_string)
                .unwrap_or_default();
            if rule.trim().is_empty() {
                return None;
            }
            let source = yaml_mapping_value(value, "source")
                .and_then(yaml_scalar_to_string)
                .filter(|source| !source.trim().is_empty());
            Some(RuleItem { id, rule, source })
        }
        _ => yaml_scalar_to_string(value)
            .filter(|rule| !rule.trim().is_empty())
            .map(|rule| RuleItem {
                id: fallback_id,
                rule,
                source: None,
            }),
    }
}

pub(crate) fn patch_rules_category_content(
    content: &str,
    category: &str,
    rules: &RulesByCategory,
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let category_block = render_rule_category_block(
        category,
        rules.get(category).map(Vec::as_slice).unwrap_or(&[]),
    );
    let mut output = String::new();
    let lines = frontmatter.split_inclusive('\n').collect::<Vec<_>>();
    let (mut index, mut in_rules, mut saw_rules, mut replaced) = (0, false, false, false);
    while index < lines.len() {
        let raw = lines[index];
        let line = raw.trim_end_matches('\n').trim_end_matches('\r');
        if !in_rules {
            if frontmatter_line_key(line) == Some("rules") {
                if line
                    .split_once(':')
                    .map(|(_, value)| value.trim())
                    .unwrap_or("")
                    .is_empty()
                {
                    output.push_str(raw);
                } else {
                    output.push_str("rules:\n");
                }
                in_rules = true;
                saw_rules = true;
            } else {
                output.push_str(raw);
            }
            index += 1;
            continue;
        }
        if is_top_level_frontmatter_boundary(line) {
            if !replaced {
                output.push_str(&category_block);
                replaced = true;
            }
            in_rules = false;
            output.push_str(raw);
            index += 1;
            continue;
        }
        if rule_category_key(line) == Some(category) {
            output.push_str(&category_block);
            replaced = true;
            index += 1;
            while index < lines.len() {
                let skipped = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                if is_top_level_frontmatter_boundary(skipped)
                    || rule_category_key(skipped).is_some()
                {
                    break;
                }
                index += 1;
            }
            continue;
        }
        output.push_str(raw);
        index += 1;
    }
    if in_rules && !replaced {
        output.push_str(&category_block);
    }
    if !saw_rules {
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&render_rules_block(rules));
    }
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(format!("---\n{}---\n{}", output, body))
}

fn render_rules_block(rules: &RulesByCategory) -> String {
    let mut output = String::from("rules:\n");
    for category in RULE_CATEGORIES {
        output.push_str(&render_rule_category_block(
            category,
            rules.get(category).map(Vec::as_slice).unwrap_or(&[]),
        ));
    }
    output
}
fn render_rule_category_block(category: &str, items: &[RuleItem]) -> String {
    let mut lines = Vec::new();
    if items.is_empty() {
        lines.push(format!("  {category}: []"));
    } else {
        lines.push(format!("  {category}:"));
        for item in items {
            lines.push(format!("    - id: {}", item.id));
            lines.push(format!("      rule: {}", yaml_double_quote(&item.rule)));
            if let Some(source) = item.source.as_deref() {
                lines.push(format!("      source: {}", yaml_double_quote(source)));
            }
        }
    }
    lines.push(String::new());
    lines.join("\n")
}
fn rule_category_key(line: &str) -> Option<&str> {
    if line.chars().take_while(|ch| *ch == ' ').count() != 2 || line.starts_with('\t') {
        return None;
    }
    let (key, _) = line.trim().split_once(':')?;
    RULE_CATEGORIES.contains(&key).then_some(key)
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
    write_atomic(&path, &format!("---\n{frontmatter}---\n{}", rule.text))
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
