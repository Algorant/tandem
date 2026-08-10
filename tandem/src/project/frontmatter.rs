//! Byte-preserving Markdown frontmatter patches for concrete project files.

use std::collections::BTreeMap;

use crate::project::split_frontmatter;
use crate::protocol::accord::AccordRecord;
use crate::protocol::workflow::CompletionRecord;
use crate::CliError;

/// Replaces only selected top-level YAML fields, retaining unknown source and
/// the Markdown body exactly as written.
pub(crate) fn patch_frontmatter_content(
    content: &str,
    updates: &BTreeMap<String, String>,
    removes: &[&str],
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let mut seen = BTreeMap::<String, bool>::new();
    let mut output_frontmatter = String::new();
    let lines = frontmatter.split_inclusive('\n').collect::<Vec<_>>();
    let mut index = 0;

    while index < lines.len() {
        let raw_line = lines[index];
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
        if let Some(key) = frontmatter_line_key(line) {
            if removes.contains(&key) {
                index += 1;
                while index < lines.len() {
                    let next = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                    if is_top_level_frontmatter_boundary(next) {
                        break;
                    }
                    index += 1;
                }
                continue;
            }
            if let Some(value) = updates.get(key) {
                output_frontmatter.push_str(&format!("{key}: {}\n", yaml_value_for_update(value)));
                seen.insert(key.to_string(), true);
                index += 1;
                while index < lines.len() {
                    let next = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                    if is_top_level_frontmatter_boundary(next) {
                        break;
                    }
                    index += 1;
                }
                continue;
            }
        }
        output_frontmatter.push_str(raw_line);
        index += 1;
    }

    if !output_frontmatter.is_empty() && !output_frontmatter.ends_with('\n') {
        output_frontmatter.push('\n');
    }
    for (key, value) in updates {
        if !seen.contains_key(key) {
            output_frontmatter.push_str(&format!("{key}: {}\n", yaml_value_for_update(value)));
        }
    }

    Ok(format!("---\n{}---\n{}", output_frontmatter, body))
}

/// Replaces the canonical Papercut resolution block while preserving unknown
/// frontmatter and the Markdown body exactly.
pub(crate) fn patch_papercut_resolution_content(
    content: &str,
    note: &str,
    resolved_at: &str,
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let block = format!(
        "resolution:\n  note: {}\n  resolvedAt: {}\n",
        yaml_double_quote(note),
        yaml_double_quote(resolved_at)
    );
    let mut output = String::new();
    let lines = frontmatter.split_inclusive('\n').collect::<Vec<_>>();
    let mut index = 0;
    let mut replaced = false;
    while index < lines.len() {
        let raw = lines[index];
        let line = raw.trim_end_matches('\n').trim_end_matches('\r');
        if frontmatter_line_key(line) == Some("resolution") {
            output.push_str(&block);
            replaced = true;
            index += 1;
            while index < lines.len() {
                let next = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                if is_top_level_frontmatter_boundary(next) {
                    break;
                }
                index += 1;
            }
            continue;
        }
        output.push_str(raw);
        index += 1;
    }
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    if !replaced {
        output.push_str(&block);
    }
    Ok(format!("---\n{}---\n{}", output, body))
}

/// Replaces the canonical accord block while preserving unrelated source bytes.
pub(crate) fn patch_accord_content(
    content: &str,
    accord: &AccordRecord,
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let accord_block = render_accord_block(accord);
    let mut output = String::new();
    let lines = frontmatter.split_inclusive('\n').collect::<Vec<_>>();
    let mut index = 0;
    let mut replaced = false;
    while index < lines.len() {
        let raw = lines[index];
        let line = raw.trim_end_matches('\n').trim_end_matches('\r');
        if frontmatter_line_key(line) == Some("accord") {
            output.push_str(&accord_block);
            replaced = true;
            index += 1;
            while index < lines.len() {
                let next = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                if is_top_level_frontmatter_boundary(next) {
                    break;
                }
                index += 1;
            }
        } else {
            output.push_str(raw);
            index += 1;
        }
    }
    if !replaced {
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&accord_block);
    }
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(format!("---\n{}---\n{}", output, body))
}

/// Replaces legacy completion fields or the canonical completion block while preserving unrelated bytes.
pub(crate) fn patch_completion_content(
    content: &str,
    completion: &CompletionRecord,
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let completion_block = render_completion_block(completion);
    let mut output = String::new();
    let lines = frontmatter.split_inclusive('\n').collect::<Vec<_>>();
    let mut index = 0;
    let mut replaced = false;
    while index < lines.len() {
        let raw = lines[index];
        let line = raw.trim_end_matches('\n').trim_end_matches('\r');
        if matches!(
            frontmatter_line_key(line),
            Some("completionSummary")
                | Some("completionValidation")
                | Some("completionReviewer")
                | Some("filesChanged")
        ) {
            index += 1;
            continue;
        }
        if frontmatter_line_key(line) == Some("completion") {
            output.push_str(&completion_block);
            replaced = true;
            index += 1;
            while index < lines.len() {
                let next = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                if is_top_level_frontmatter_boundary(next) {
                    break;
                }
                index += 1;
            }
        } else {
            output.push_str(raw);
            index += 1;
        }
    }
    if !replaced {
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&completion_block);
    }
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(format!("---\n{}---\n{}", output, body))
}

fn render_completion_block(completion: &CompletionRecord) -> String {
    let mut lines = vec!["completion:".to_string()];
    if let Some(outcome) = completion.outcome.as_deref() {
        lines.push(format!("  outcome: {}", yaml_double_quote(outcome)));
    }
    lines.push(format!(
        "  summary: {}",
        yaml_double_quote(&completion.summary)
    ));
    push_nested_array_line(&mut lines, "filesChanged", &completion.files_changed);
    push_optional_nested_line(&mut lines, "validation", completion.validation.as_deref());
    push_optional_nested_line(&mut lines, "reviewer", completion.reviewer.as_deref());
    lines.push(String::new());
    lines.join("\n")
}

fn render_accord_block(accord: &AccordRecord) -> String {
    let mut lines = vec![
        "accord:".to_string(),
        format!("  status: {}", yaml_double_quote(&accord.status)),
    ];
    push_optional_nested_line(&mut lines, "assignee", accord.assignee.as_deref());
    push_optional_nested_line(&mut lines, "claimedAt", accord.claimed_at.as_deref());
    push_optional_nested_line(&mut lines, "deliveredAt", accord.delivered_at.as_deref());
    push_nested_array_line(&mut lines, "deliverables", &accord.deliverables);
    if !accord.validations.is_empty() {
        lines.push("  validation:".to_string());
        lines.push(format!(
            "    commands: {}",
            inline_array(&accord.validations)
        ));
    }
    push_nested_array_line(&mut lines, "constraints", &accord.constraints);
    push_optional_nested_line(&mut lines, "summary", accord.summary.as_deref());
    push_nested_array_line(&mut lines, "evidence", &accord.evidence);
    push_nested_array_line(&mut lines, "filesChanged", &accord.files_changed);
    push_optional_nested_line(&mut lines, "reviewer", accord.reviewer.as_deref());
    push_optional_nested_line(&mut lines, "note", accord.note.as_deref());
    push_optional_nested_line(&mut lines, "reason", accord.reason.as_deref());
    lines.push(format!(
        "  updatedAt: {}",
        yaml_double_quote(&accord.updated_at)
    ));
    lines.push(String::new());
    lines.join("\n")
}

fn inline_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| yaml_double_quote(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn push_optional_nested_line(lines: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(format!("  {key}: {}", yaml_double_quote(value.trim())));
    }
}

fn push_nested_array_line(lines: &mut Vec<String>, key: &str, values: &[String]) {
    if !values.is_empty() {
        lines.push(format!("  {key}: {}", inline_array(values)));
    }
}

pub(crate) fn replace_markdown_body(content: &str, body: &str) -> Result<String, CliError> {
    let (frontmatter, _) = split_frontmatter(content).map_err(CliError::user)?;
    Ok(format!("---\n{}---\n{}", frontmatter, body))
}

pub(crate) fn frontmatter_line_key(line: &str) -> Option<&str> {
    if line.starts_with(' ') || line.starts_with('\t') || line.trim_start().starts_with('-') {
        return None;
    }
    let (key, _) = line.split_once(':')?;
    let key = key.trim();
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

pub(crate) fn is_top_level_frontmatter_boundary(line: &str) -> bool {
    !line.starts_with(' ')
        && !line.starts_with('\t')
        && !line.trim().is_empty()
        && (frontmatter_line_key(line).is_some() || line.trim_start().starts_with('#'))
}

pub(crate) fn yaml_double_quote(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('\"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}

fn yaml_value_for_update(value: &str) -> String {
    if value.starts_with('[') && value.ends_with(']') {
        value.to_string()
    } else {
        yaml_double_quote(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches_accord_without_touching_body_or_other_fields() {
        let input = "---\nid: task-1\ntitle: Demo\naccord:\n  status: ready\n  assignee: pi\nreview:\n  status: pending\n---\n\nBody\n";
        let accord = AccordRecord {
            status: "delivered".to_string(),
            assignee: Some("pi".to_string()),
            delivered_at: Some("2026-06-26T00:00:00Z".to_string()),
            summary: Some("Done".to_string()),
            validations: vec!["cargo test".to_string()],
            evidence: vec!["cargo test passed".to_string()],
            updated_at: "2026-06-26T00:00:00Z".to_string(),
            ..AccordRecord::default()
        };
        let output = patch_accord_content(input, &accord).unwrap();
        assert!(output.contains("accord:\n  status: \"delivered\"\n"));
        assert!(output.contains("  assignee: \"pi\"\n"));
        assert!(output.contains("  deliveredAt: \"2026-06-26T00:00:00Z\"\n"));
        assert!(output.contains("  validation:\n    commands: [\"cargo test\"]\n"));
        assert!(output.contains("  summary: \"Done\"\n"));
        assert!(output.contains("  evidence: [\"cargo test passed\"]\n"));
        assert!(output.contains("review:\n  status: pending\n"));
        assert!(output.ends_with("\nBody\n"));
    }

    #[test]
    fn patches_completion_as_nested_metadata_and_preserves_body() {
        let input = "---\nid: task-1\ntype: task\ntitle: Demo\ncompletionSummary: old\nfilesChanged: [old.rs]\n---\n\nBody\n";
        let output = patch_completion_content(
            input,
            &CompletionRecord {
                summary: "Done".to_string(),
                files_changed: vec!["src/main.rs".to_string()],
                validation: Some("cargo test passed".to_string()),
                reviewer: Some("Algorant".to_string()),
                ..CompletionRecord::default()
            },
        )
        .unwrap();
        assert!(!output.contains("completionSummary:"));
        assert!(!output.contains("filesChanged: [old.rs]"));
        assert!(output.contains("completion:\n  summary: \"Done\"\n"));
        assert!(output.contains("  filesChanged: [\"src/main.rs\"]\n"));
        assert!(output.contains("  validation: \"cargo test passed\"\n"));
        assert!(output.contains("  reviewer: \"Algorant\"\n"));
        assert!(output.ends_with("\nBody\n"));
    }

    #[test]
    fn patches_one_field_without_touching_unknown_source_or_body() {
        let input = "---\nid: task-1\nunknown: { nested: keep }\ntitle: old\n---\n# body\n";
        let output = patch_frontmatter_content(
            input,
            &BTreeMap::from([(String::from("title"), String::from("new"))]),
            &[],
        )
        .unwrap();
        assert_eq!(
            output,
            "---\nid: task-1\nunknown: { nested: keep }\ntitle: \"new\"\n---\n# body\n"
        );
    }
}
