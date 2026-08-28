---
id: task-244
type: task
title: "Cache the Logs filter result and stop rebuilding detail lines to count them"
state: todo
priority: "high"
effort: "small"
references: ["task-233", "task-226"]
relatedFiles: ["tandem/src/tui/logs.rs", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs"]
tags: ["tui", "logs", "performance"]
createdAt: "2026-08-28T23:33:30Z"
updatedAt: "2026-08-28T23:33:30Z"
---

## Description


Fix the Logs screen search slowdown diagnosed in task-233. Preserve current behavior exactly; this is a performance change, not a product change.

## Cause

See task-233 for full findings. Summary:

- `filter_logs` runs `log_matches_query` over all 257 logs on any non-empty query.
- `log_matches_query` builds a `String` containing each document's whole body, then allocates a second full lowercase copy, costing roughly 2.5 MB of allocation per pass on the real workspace.
- `filtered_logs()` has 8 call sites and caches nothing, so one keystroke triggers roughly five filter passes and two full detail-line builds.

## Scope

1. Cache the filtered log list. Recompute only when `logs` or `log_search_filter` changes, not per accessor. `reload.rs` already owns the reload path where `logs` is replaced, so invalidation has a clear home.
2. Stop re-lowercasing the query per document at `logs.rs:131`. The caller already lowercased it at `logs.rs:79`.
3. Avoid the double full-text allocation in `log_matches_query`. Match against the constituent fields and body directly instead of concatenating into one `String` and then lowercasing the whole thing.
4. Make `log_detail_line_count` count lines without constructing the full `Vec<Line>`, or reuse an already-built detail-line result.
5. Make `selected_log()` stop building the entire filtered Vec to take one element.

## Out of scope

- Board-side equivalents of the same pattern. `detail_line_count` at `state.rs:968` has the same build-to-count shape, but the Board corpus is far smaller and it is not the reported problem. Note it, do not fix it here.
- Any change to what search matches. The set of fields searched must stay identical, including hierarchy role, relationship, and parent title.
- New dependencies. None are needed.

## Acceptance

- Typing in the Logs search box is responsive on the real 257-document workspace.
- Existing `logs.rs` tests still pass unchanged, including `filters_logs_by_id_title_summary_and_body`, `logs_render_and_filter_canonical_epic_task_and_subtask_context`, and `canceled_logs_render_and_filter_as_canceled_not_completed`.
- A test covers that the cached filter result is invalidated when the filter text changes and when logs reload.
- `cargo clippy --all-targets --all-features -- -D warnings` is clean.

