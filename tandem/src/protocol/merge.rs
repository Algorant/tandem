//! Three-way semantic merge of Tandem record files.
//!
//! Sync merges records by meaning instead of by line. Frontmatter is compared
//! field by field (one level into nested maps such as `accord`), set-like
//! lists are merged as sets, `updatedAt` takes the later value, and Markdown
//! bodies use a caller-supplied three-way text merge. Anything else changed
//! differently on both sides is a conflict; this module never picks a winner
//! by time and never emits conflict markers.

use yaml_rust2::{Yaml, YamlLoader};

/// Lists whose order carries no meaning and whose entries merge as sets.
const SET_FIELDS: &[&str] = &[
    "tags",
    "references",
    "blockers",
    "relatedFiles",
    "supersedes",
    "deciders",
    "filesChanged",
];

/// Result of merging one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Merged {
    Clean(String),
    /// The named fields changed differently on both sides.
    Conflict(Vec<String>),
}

/// Merges one Markdown record (YAML frontmatter plus body).
///
/// `text_merge(base, local, remote)` must return the merged text or `None`
/// when the three versions do not merge cleanly.
pub(crate) fn merge_markdown(
    base: &str,
    local: &str,
    remote: &str,
    text_merge: &dyn Fn(&str, &str, &str) -> Option<String>,
) -> Merged {
    if local == base || local == remote {
        return Merged::Clean(remote.to_string());
    }
    if remote == base {
        return Merged::Clean(local.to_string());
    }
    let (Some(b), Some(l), Some(r)) = (split(base), split(local), split(remote)) else {
        return Merged::Conflict(vec!["frontmatter".to_string()]);
    };
    let mut conflicts = Vec::new();
    let prefix = pick(&b.prefix, &l.prefix, &r.prefix).unwrap_or_else(|| {
        conflicts.push("frontmatter".to_string());
        r.prefix.clone()
    });
    let blocks = merge_blocks(&b.blocks, &l.blocks, &r.blocks, 0, &mut conflicts);
    let body = match pick(&b.body, &l.body, &r.body) {
        Some(body) => body,
        None => match text_merge(&b.body, &l.body, &r.body) {
            Some(body) => body,
            None => {
                conflicts.push("body".to_string());
                r.body.clone()
            }
        },
    };
    if !conflicts.is_empty() {
        return Merged::Conflict(conflicts);
    }
    let mut output = String::from("---\n");
    output.push_str(&prefix);
    for block in blocks {
        output.push_str(&block.text);
        if !block.text.ends_with('\n') {
            output.push('\n');
        }
    }
    output.push_str("---\n");
    output.push_str(&body);
    Merged::Clean(output)
}

/// Merges one per-actor event ledger. Each ledger has exactly one writer, so
/// one side must extend the other exactly; anything else means two writers
/// shared an identity and is reported rather than combined.
pub(crate) fn merge_ledger(local: &str, remote: &str) -> Merged {
    if local.starts_with(remote) {
        Merged::Clean(local.to_string())
    } else if remote.starts_with(local) {
        Merged::Clean(remote.to_string())
    } else {
        Merged::Conflict(vec!["events".to_string()])
    }
}

/// Returns the three-way pick when at most one side changed.
fn pick<T: PartialEq + Clone>(base: &T, local: &T, remote: &T) -> Option<T> {
    if local == base || local == remote {
        Some(remote.clone())
    } else if remote == base {
        Some(local.clone())
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Block {
    key: String,
    text: String,
}

struct Split {
    prefix: String,
    blocks: Vec<Block>,
    body: String,
}

fn split(content: &str) -> Option<Split> {
    let rest = content.strip_prefix("---\n")?;
    let mut offset = 0;
    let mut frontmatter_end = None;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\n', '\r']).trim() == "---" {
            frontmatter_end = Some((offset, offset + line.len()));
            break;
        }
        offset += line.len();
    }
    let (end, body_start) = frontmatter_end?;
    let (prefix, blocks) = split_blocks(&rest[..end], 0);
    Some(Split {
        prefix,
        blocks,
        body: rest[body_start..].to_string(),
    })
}

/// Splits YAML mapping text into key blocks at the given indentation.
fn split_blocks(text: &str, indent: usize) -> (String, Vec<Block>) {
    let mut prefix = String::new();
    let mut blocks: Vec<Block> = Vec::new();
    for line in text.split_inclusive('\n') {
        match block_key(line, indent) {
            Some(key) => blocks.push(Block {
                key: key.to_string(),
                text: line.to_string(),
            }),
            None => match blocks.last_mut() {
                Some(block) => block.text.push_str(line),
                None => prefix.push_str(line),
            },
        }
    }
    (prefix, blocks)
}

fn block_key(line: &str, indent: usize) -> Option<&str> {
    let (spaces, rest) = line.split_at_checked(indent)?;
    if !spaces.bytes().all(|byte| byte == b' ') || rest.starts_with([' ', '\t', '-', '#']) {
        return None;
    }
    let (key, _) = rest.split_once(':')?;
    let key = key.trim();
    (!key.is_empty() && !key.contains(' ')).then_some(key)
}

fn merge_blocks(
    base: &[Block],
    local: &[Block],
    remote: &[Block],
    depth: usize,
    conflicts: &mut Vec<String>,
) -> Vec<Block> {
    let find = |blocks: &[Block], key: &str| {
        blocks
            .iter()
            .find(|block| block.key == key)
            .map(|block| block.text.clone())
    };
    let mut keys: Vec<String> = local.iter().map(|block| block.key.clone()).collect();
    for block in remote {
        if !keys.contains(&block.key) {
            keys.push(block.key.clone());
        }
    }
    let mut output = Vec::new();
    for key in keys {
        let (b, l, r) = (find(base, &key), find(local, &key), find(remote, &key));
        if let Some(text) = merge_value(&key, b, l, r, depth, conflicts) {
            output.push(Block { key, text });
        }
    }
    output
}

fn merge_value(
    key: &str,
    base: Option<String>,
    local: Option<String>,
    remote: Option<String>,
    depth: usize,
    conflicts: &mut Vec<String>,
) -> Option<String> {
    if let Some(value) = pick(&base, &local, &remote) {
        return value;
    }
    let (Some(l), Some(r)) = (local.as_deref(), remote.as_deref()) else {
        // Removed on one side and changed on the other.
        conflicts.push(key.to_string());
        return remote;
    };
    if key == "updatedAt" {
        return Some(if scalar(l) >= scalar(r) { l } else { r }.to_string());
    }
    if SET_FIELDS.contains(&key) {
        if let (Some(lv), Some(rv)) = (list(l, depth), list(r, depth)) {
            let bv = base
                .as_deref()
                .and_then(|text| list(text, depth))
                .unwrap_or_default();
            return Some(render_list(key, depth, &merge_set(&bv, &lv, &rv)));
        }
    }
    if depth == 0 {
        if let (Some(lm), Some(rm)) = (nested(l), nested(r)) {
            let bm = base.as_deref().and_then(nested);
            if lm.0 == rm.0 && bm.as_ref().is_none_or(|bm| bm.0 == lm.0) {
                let base_blocks = bm.map(|bm| bm.1).unwrap_or_default();
                let mut nested_conflicts = Vec::new();
                let merged = merge_blocks(&base_blocks, &lm.1, &rm.1, 1, &mut nested_conflicts);
                if nested_conflicts.is_empty() {
                    let mut text = lm.0.clone();
                    for block in merged {
                        text.push_str(&block.text);
                        if !block.text.ends_with('\n') {
                            text.push('\n');
                        }
                    }
                    return Some(text);
                }
                conflicts.extend(
                    nested_conflicts
                        .into_iter()
                        .map(|field| format!("{key}.{field}")),
                );
                return remote;
            }
        }
    }
    conflicts.push(key.to_string());
    remote
}

/// Splits `key:\n  sub: value...` into its header line and sub-blocks.
fn nested(text: &str) -> Option<(String, Vec<Block>)> {
    let (header, rest) = text.split_once('\n')?;
    if !header.trim_end().ends_with(':') {
        return None;
    }
    let (prefix, blocks) = split_blocks(rest, 2);
    (prefix.trim().is_empty() && !blocks.is_empty()).then(|| (format!("{header}\n"), blocks))
}

fn scalar(text: &str) -> String {
    let value = text.split_once(':').map(|(_, value)| value).unwrap_or("");
    value.trim().trim_matches('"').to_string()
}

fn list(text: &str, depth: usize) -> Option<Vec<String>> {
    let dedented: String = text
        .split_inclusive('\n')
        .map(|line| line.get(depth * 2..).unwrap_or(line.trim_start()))
        .collect();
    let docs = YamlLoader::load_from_str(&dedented).ok()?;
    let hash = docs.first()?.as_hash()?;
    let (_, value) = hash.iter().next()?;
    match value {
        Yaml::Array(items) => items
            .iter()
            .map(|item| match item {
                Yaml::String(value) | Yaml::Real(value) => Some(value.clone()),
                Yaml::Integer(value) => Some(value.to_string()),
                Yaml::Boolean(value) => Some(value.to_string()),
                _ => None,
            })
            .collect(),
        Yaml::Null => Some(Vec::new()),
        _ => None,
    }
}

fn merge_set(base: &[String], local: &[String], remote: &[String]) -> Vec<String> {
    let mut output = Vec::new();
    for item in local.iter().chain(remote) {
        let keep = (local.contains(item) && remote.contains(item)) || !base.contains(item);
        if keep && !output.contains(item) {
            output.push(item.clone());
        }
    }
    output
}

fn render_list(key: &str, depth: usize, items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| quote(item)).collect();
    format!("{}{key}: [{}]\n", " ".repeat(depth * 2), quoted.join(", "))
}

fn quote(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_text_merge(_: &str, _: &str, _: &str) -> Option<String> {
        None
    }

    const BASE: &str = "---\nid: task-1\nuid: u\ntitle: \"Base\"\npriority: \"low\"\ntags: [\"a\"]\naccord:\n  status: \"ready\"\n  acceptance: [\"x\"]\n  updatedAt: \"2026-01-01T00:00:00Z\"\nupdatedAt: \"2026-01-01T00:00:00Z\"\n---\n\nBody\n";

    #[test]
    fn combines_independent_fields_sets_and_timestamps() {
        let local = BASE
            .replace("\"low\"", "\"high\"")
            .replace("[\"a\"]", "[\"a\", \"l\"]")
            .replace(
                "updatedAt: \"2026-01-01T00:00:00Z\"\n---",
                "updatedAt: \"2026-01-02T00:00:00Z\"\n---",
            );
        let remote = BASE
            .replace("\"Base\"", "\"Remote\"")
            .replace("[\"a\"]", "[\"r\"]")
            .replace("status: \"ready\"", "status: \"claimed\"")
            .replace(
                "updatedAt: \"2026-01-01T00:00:00Z\"\n---",
                "updatedAt: \"2026-01-03T00:00:00Z\"\n---",
            );
        let Merged::Clean(merged) = merge_markdown(BASE, &local, &remote, &no_text_merge) else {
            panic!("expected clean merge");
        };
        assert!(merged.contains("title: \"Remote\"\n"));
        assert!(merged.contains("priority: \"high\"\n"));
        assert!(merged.contains("tags: [\"l\", \"r\"]\n"));
        assert!(merged.contains("  status: \"claimed\"\n"));
        assert!(merged.contains("updatedAt: \"2026-01-03T00:00:00Z\"\n---"));
        assert!(merged.ends_with("\nBody\n"));
    }

    #[test]
    fn same_field_changed_differently_is_a_conflict() {
        let local = BASE.replace("\"Base\"", "\"Local\"");
        let remote = BASE
            .replace("\"Base\"", "\"Remote\"")
            .replace("status: \"ready\"", "status: \"claimed\"");
        let local = local.replace("status: \"ready\"", "status: \"blocked\"");
        assert_eq!(
            merge_markdown(BASE, &local, &remote, &no_text_merge),
            Merged::Conflict(vec!["title".to_string(), "accord.status".to_string()])
        );
    }

    #[test]
    fn bodies_use_the_text_merge_and_never_emit_markers() {
        let local = BASE.replace("Body\n", "Body\nlocal\n");
        let remote = BASE.replace("\nBody\n", "\nremote\nBody\n");
        let merged = merge_markdown(BASE, &local, &remote, &|_, _, _| {
            Some("\nremote\nBody\nlocal\n".to_string())
        });
        assert_eq!(
            merged,
            Merged::Clean(BASE.replace("\nBody\n", "\nremote\nBody\nlocal\n"))
        );
        assert_eq!(
            merge_markdown(BASE, &local, &remote, &no_text_merge),
            Merged::Conflict(vec!["body".to_string()])
        );
    }

    #[test]
    fn ledgers_merge_only_by_extension() {
        assert_eq!(
            merge_ledger("a\nb\n", "a\n"),
            Merged::Clean("a\nb\n".into())
        );
        assert_eq!(
            merge_ledger("a\n", "a\nc\n"),
            Merged::Clean("a\nc\n".into())
        );
        assert!(matches!(
            merge_ledger("a\nb\n", "a\nc\n"),
            Merged::Conflict(_)
        ));
    }
}
