---
id: task-244
type: task
title: "Cache applied Logs filter indexes in memory"
state: "in-progress"
priority: "high"
effort: "small"
references: ["task-233", "task-226"]
relatedFiles: ["tandem/src/tui/logs.rs", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs"]
tags: ["tui", "logs", "performance"]
createdAt: "2026-08-28T23:33:30Z"
updatedAt: "2026-08-29T03:27:44Z"
accord:
  status: "claimed"
  assignee: "worker-task-244-e497e28e"
  claimedAt: "2026-08-29T03:27:44Z"
  updatedAt: "2026-08-29T03:27:44Z"
assignee: "worker-task-244-e497e28e"
---

## Description

Fix the measured slowdown when navigating or redrawing the Logs view after a non-empty search filter has been applied. Preserve current search matches and UI behavior exactly.

## Corrected diagnosis

The initial task-233 diagnosis overstated the problem by claiming that each typed character runs several full searches. That is incorrect: typing changes `log_search_input`; the committed `log_search_filter` changes only when Enter is pressed (`tandem/src/tui/state.rs:369-395`).

Real release-TUI measurements against this workspace's 257 Logs showed:

| Interaction | Observed response |
| --- | ---: |
| Open Logs | 17–27 ms |
| Navigate with no filter | under 0.5 ms |
| Scroll log detail | about 3–4 ms |
| Navigate after applying `accord` | 14–281 ms, with some input backing up |

Text rendering, empty-filter navigation, and detail scrolling are already fast. The measured problem is repeated full-corpus searching after a filter is committed.

## Cause

`filter_logs` (`tandem/src/tui/logs.rs:75-131`) scans all Logs for every non-empty filter. The result is not retained. List rendering, detail selection, navigation bounds, selection clamping, and status generation can independently call `filtered_logs()`, repeating the same search during one interaction (`tandem/src/tui/state.rs:743-775, 888-895, 986-1041, 1131-1234`).

## Scope

1. Add ephemeral in-memory filtered indexes to `TuiApp`, representing positions in the existing `logs: Vec<Document>`.
2. Calculate the indexes once when a filter is committed with Enter.
3. Reuse the indexes for counts, selection, status, navigation, and rendering.
4. Recalculate after Logs reload while a filter remains active.
5. Restore the unfiltered view without running a full search when the filter is cleared.

The indexes must remain runtime-only. Do not persist them, copy Documents into a second collection, or add a filesystem cache.

## Out of scope

- Rewriting or pre-indexing the search algorithm.
- Changing which fields search matches.
- Optimizing detail-line construction or counting.
- Board-side projection or detail work.
- Background threads, async work, dependencies, protocol changes, or filesystem changes.

## Acceptance

- Search results are identical to current behavior, including ID, title, completion metadata, body, hierarchy role, relationship, and parent-title matches.
- After applying a non-empty filter, navigation and redraw reuse the stored indexes and do not invoke another full-corpus search.
- Applying, clearing, and replacing a filter updates the indexes correctly.
- Reloading Logs rebuilds active-filter indexes and preserves valid selection behavior.
- Empty-filter navigation remains correct.
- Existing Logs tests pass unchanged, with focused tests covering cache creation, replacement, clearing, reload invalidation, and indexed selection.
- `cargo fmt --check`, full tests, and `cargo clippy --all-targets --all-features -- -D warnings` pass.
