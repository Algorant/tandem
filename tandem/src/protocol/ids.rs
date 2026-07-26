//! Canonical Tandem task-ID grammar and allocation queries.
//!
//! These functions operate only on resolved document IDs. Project code owns
//! Board/Logs scanning, locks, and atomic reservations.

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
    fn allocation_query_ignores_noncanonical_ids() {
        let ids = ["task-1", "task-4", "task-04", "task-4-1", "decision-9"];
        assert_eq!(next_sequential_number(ids.iter().copied(), "task"), 4);
    }
}
