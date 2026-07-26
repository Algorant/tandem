//! Byte-preserving Markdown frontmatter patches for concrete project files.

use std::collections::BTreeMap;

use crate::project::split_frontmatter;
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
