//! Canonical Tandem task-ID grammar and allocation queries.
//!
//! These functions operate only on resolved document IDs. Project code owns
//! Board/Logs scanning, locks, and atomic reservations.

use std::cmp::Ordering;

/// Orders document IDs by prefix, then by each numeric segment as a number.
///
/// `task-2` precedes `task-10`, and `task-2-2` precedes `task-2-10`. IDs whose
/// segments are not canonical numbers sort before numbered IDs sharing their
/// prefix, ordered by raw text.
pub(crate) fn compare_ids(a: &str, b: &str) -> Ordering {
    sort_key(a).cmp(&sort_key(b))
}

fn sort_key(id: &str) -> (&str, Vec<usize>, &str) {
    let (prefix, rest) = id.split_once('-').unwrap_or((id, ""));
    let numbers = rest
        .split('-')
        .map(positive_canonical_number)
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default();
    (prefix, numbers, id)
}

/// Returns the positive numeric suffix of a global `task-N` ID.
pub(crate) fn global_task_number(id: &str) -> Option<usize> {
    id.strip_prefix("task-")
        .filter(|suffix| !suffix.contains('-'))
        .and_then(positive_canonical_number)
}

/// Returns the positive child suffix of a parent-derived `task-N-M` ID.
pub(crate) fn subtask_suffix(id: &str, parent_id: &str) -> Option<usize> {
    global_task_number(parent_id)?;
    id.strip_prefix(&format!("{parent_id}-"))
        .and_then(positive_canonical_number)
}

/// Returns the greatest allocated positive number for `prefix-N` IDs.
///
/// The caller supplies the coherent Board-and-Logs snapshot; this query never
/// touches the filesystem and does not reserve an ID.
pub(crate) fn next_sequential_number<'a>(
    ids: impl Iterator<Item = &'a str>,
    prefix: &str,
) -> usize {
    let needle = format!("{prefix}-");
    ids.filter_map(|id| id.strip_prefix(&needle))
        .filter_map(positive_canonical_number)
        .max()
        .unwrap_or(0)
}

/// Number of lowercase hex characters of a record uid used in a provisional ID.
pub(crate) const PROVISIONAL_HEX_LEN: usize = 8;

/// Returns the provisional ID `<prefix>-new-<hex>` for an unpublished record.
///
/// Provisional IDs are local display handles for records that have not yet
/// reached the shared `tandem` branch. Publication replaces them with the next
/// sequential ID; the shared branch never contains one.
pub(crate) fn provisional_id(prefix: &str, uid: &str) -> String {
    let hex: String = uid
        .chars()
        .filter(char::is_ascii_hexdigit)
        .take(PROVISIONAL_HEX_LEN)
        .collect::<String>()
        .to_ascii_lowercase();
    format!("{prefix}-new-{hex}")
}

/// Splits a provisional ID into its prefix and uid hex, e.g.
/// `task-new-3f9a2c1d` into (`task`, `3f9a2c1d`).
pub(crate) fn provisional_parts(id: &str) -> Option<(&str, &str)> {
    let (prefix, hex) = id.rsplit_once("-new-")?;
    let valid = !prefix.is_empty()
        && !prefix.contains("-new-")
        && hex.len() == PROVISIONAL_HEX_LEN
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    valid.then_some((prefix, hex))
}

pub(crate) fn is_provisional(id: &str) -> bool {
    provisional_parts(id).is_some()
}

/// What the publication numbering pass needs to know about one record.
#[derive(Debug, Clone)]
pub(crate) struct NumberingRecord {
    pub(crate) id: String,
    pub(crate) kind: NumberingKind,
    /// Creation time; provisional records are numbered in creation order.
    pub(crate) created_at: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum NumberingKind {
    /// A Task-type record. `subtask` is true when its parent is a normal Task,
    /// which makes its ID parent-derived (`task-N-M`).
    Task {
        parent: Option<String>,
        subtask: bool,
    },
    Decision,
    Rule {
        category: String,
    },
}

/// Assigns permanent sequential IDs to every provisional record.
///
/// Numbers continue after the highest existing number in `records`, which
/// must be the complete merged board (active records and Logs). Parents are
/// numbered before their Subtasks so a Subtask derives from its parent's final
/// ID. Returns `(provisional, permanent)` pairs in assignment order.
pub(crate) fn assign_sequential_ids(records: &[NumberingRecord]) -> Vec<(String, String)> {
    let mut pending: Vec<&NumberingRecord> =
        records.iter().filter(|r| is_provisional(&r.id)).collect();
    pending.sort_by(|a, b| {
        a.created_at
            .as_deref()
            .unwrap_or("")
            .cmp(b.created_at.as_deref().unwrap_or(""))
            .then_with(|| a.id.cmp(&b.id))
    });
    let ids: Vec<&str> = records.iter().map(|r| r.id.as_str()).collect();
    let mut taken: std::collections::HashSet<String> =
        ids.iter().map(|id| id.to_string()).collect();
    let mut renames: Vec<(String, String)> = Vec::new();
    let mut resolved: std::collections::HashMap<String, String> = Default::default();
    let next_in = |prefix: &str, taken: &std::collections::HashSet<String>| -> String {
        let mut next = next_sequential_number(taken.iter().map(String::as_str), prefix) + 1;
        loop {
            let candidate = format!("{prefix}-{next}");
            if !taken.contains(&candidate) {
                return candidate;
            }
            next += 1;
        }
    };
    // Global Tasks, Decisions, and Rules first; Subtasks after their parents.
    for pass_subtasks in [false, true] {
        for record in &pending {
            let new_id = match &record.kind {
                NumberingKind::Task { subtask, parent } => {
                    if *subtask != pass_subtasks {
                        continue;
                    }
                    if *subtask {
                        let parent = parent.as_deref().unwrap_or("task");
                        let parent = resolved
                            .get(parent)
                            .cloned()
                            .unwrap_or_else(|| parent.to_string());
                        next_in(&parent, &taken)
                    } else {
                        next_in("task", &taken)
                    }
                }
                NumberingKind::Decision if !pass_subtasks => next_in("decision", &taken),
                NumberingKind::Rule { category } if !pass_subtasks => next_in(category, &taken),
                _ => continue,
            };
            taken.insert(new_id.clone());
            resolved.insert(record.id.clone(), new_id.clone());
            renames.push((record.id.clone(), new_id));
        }
    }
    renames
}

/// Replaces every whole-token occurrence of `old` with `new`.
///
/// A token boundary means the match is not directly preceded by an ID
/// character and not followed by an alphanumeric character or a `-<digit>`
/// continuation, so replacing `task-2` never touches `task-20` or
/// `task-2-1`.
pub(crate) fn replace_id_token(text: &str, old: &str, new: &str) -> String {
    if old.is_empty() || !text.contains(old) {
        return text.to_string();
    }
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    while let Some(found) = text[cursor..].find(old) {
        let start = cursor + found;
        let end = start + old.len();
        let before_ok = start == 0 || {
            let previous = bytes[start - 1];
            !(previous.is_ascii_alphanumeric() || previous == b'-' || previous == b'_')
        };
        let after_ok = match bytes.get(end) {
            None => true,
            Some(next) if next.is_ascii_alphanumeric() || *next == b'_' => false,
            Some(b'-') => !bytes.get(end + 1).is_some_and(u8::is_ascii_alphanumeric),
            Some(_) => true,
        };
        output.push_str(&text[cursor..start]);
        output.push_str(if before_ok && after_ok { new } else { old });
        cursor = end;
    }
    output.push_str(&text[cursor..]);
    output
}

fn positive_canonical_number(value: &str) -> Option<usize> {
    if value.is_empty()
        || value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    value.parse::<usize>().ok().filter(|number| *number > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_only_canonical_global_and_parent_derived_ids() {
        assert_eq!(global_task_number("task-12"), Some(12));
        assert_eq!(global_task_number("task-12-1"), None);
        assert_eq!(global_task_number("task-01"), None);
        assert_eq!(subtask_suffix("task-12-3", "task-12"), Some(3));
        assert_eq!(subtask_suffix("task-12-03", "task-12"), None);
    }

    #[test]
    fn orders_numeric_segments_as_numbers() {
        let mut ids = vec![
            "task-10",
            "task-2",
            "task-1",
            "decision-8",
            "task-2-10",
            "task-2-2",
            "decision-10",
        ];
        ids.sort_by(|a, b| compare_ids(a, b));
        assert_eq!(
            ids,
            vec![
                "decision-8",
                "decision-10",
                "task-1",
                "task-2",
                "task-2-2",
                "task-2-10",
                "task-10",
            ]
        );
    }

    #[test]
    fn orders_noncanonical_ids_before_numbered_ids_of_the_same_prefix() {
        let mut ids = vec!["task-2", "task-04", "task", "task-1"];
        ids.sort_by(|a, b| compare_ids(a, b));
        assert_eq!(ids, vec!["task", "task-04", "task-1", "task-2"]);
    }

    #[test]
    fn provisional_ids_round_trip_and_reject_lookalikes() {
        let id = provisional_id("task", "3F9A2C1D-0000-4000-8000-000000000000");
        assert_eq!(id, "task-new-3f9a2c1d");
        assert_eq!(provisional_parts(&id), Some(("task", "3f9a2c1d")));
        assert_eq!(
            provisional_parts("always-new-0123abcd"),
            Some(("always", "0123abcd"))
        );
        assert!(!is_provisional("task-12"));
        assert!(!is_provisional("task-new-xyz"));
        assert!(!is_provisional("task-new-3f9a2c1"));
    }

    #[test]
    fn numbering_continues_sequences_and_numbers_parents_before_subtasks() {
        let task = |id: &str, parent: Option<&str>, subtask: bool, at: &str| NumberingRecord {
            id: id.to_string(),
            kind: NumberingKind::Task {
                parent: parent.map(str::to_string),
                subtask,
            },
            created_at: Some(at.to_string()),
        };
        let records = vec![
            task("task-49", None, false, "1"),
            task("task-49-2", Some("task-49"), true, "1"),
            task("task-new-bbbbbbbb", Some("task-new-aaaaaaaa"), true, "3"),
            task("task-new-aaaaaaaa", None, false, "2"),
            task("task-new-cccccccc", Some("task-49"), true, "4"),
            NumberingRecord {
                id: "decision-new-dddddddd".into(),
                kind: NumberingKind::Decision,
                created_at: None,
            },
            NumberingRecord {
                id: "always-4".into(),
                kind: NumberingKind::Rule {
                    category: "always".into(),
                },
                created_at: None,
            },
            NumberingRecord {
                id: "always-new-eeeeeeee".into(),
                kind: NumberingKind::Rule {
                    category: "always".into(),
                },
                created_at: None,
            },
        ];
        let renames: std::collections::HashMap<_, _> =
            assign_sequential_ids(&records).into_iter().collect();
        assert_eq!(renames["task-new-aaaaaaaa"], "task-50");
        assert_eq!(renames["task-new-bbbbbbbb"], "task-50-1");
        assert_eq!(renames["task-new-cccccccc"], "task-49-3");
        assert_eq!(renames["decision-new-dddddddd"], "decision-1");
        assert_eq!(renames["always-new-eeeeeeee"], "always-5");
    }

    #[test]
    fn token_replacement_respects_id_boundaries() {
        let text = "see task-2, task-20, task-2-1 and (task-2).\ntask-2";
        assert_eq!(
            replace_id_token(text, "task-2", "task-9"),
            "see task-9, task-20, task-2-1 and (task-9).\ntask-9"
        );
        assert_eq!(
            replace_id_token("[\"task-new-aaaaaaaa\"]", "task-new-aaaaaaaa", "task-5"),
            "[\"task-5\"]"
        );
    }

    #[test]
    fn allocation_query_ignores_noncanonical_ids() {
        let ids = ["task-1", "task-4", "task-04", "task-4-1", "decision-9"];
        assert_eq!(next_sequential_number(ids.iter().copied(), "task"), 4);
    }
}
