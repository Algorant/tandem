---
id: task-233
type: task
title: "Research remaining Logs TUI slowdown and optimization opportunities"
priority: "high"
effort: "medium"
references: ["task-226", "task-242", "task-244"]
relatedFiles: ["tandem/src/tui/logs.rs", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/mod.rs"]
tags: ["tui", "logs", "performance", "research"]
createdAt: "2026-08-21T03:29:46Z"
updatedAt: "2026-08-28T23:33:37Z"
completedAt: "2026-08-28T23:33:37Z"
completion:
  summary: "Diagnosed the Logs TUI slowdown by reading the filter path against the real 257-document workspace. Cause is log_matches_query allocating the full document body twice per document on any non-empty search, combined with filtered_logs() being recomputed roughly five times per keystroke and log_detail_line_count building the entire detail-line vector only to read its length. The empty-filter fast path is free, which is why task-226's benchmark reported 55 ms and missed this. Concluded no benchmark harness or PTY dependency is needed. Fix opened as task-244."
---

## Description

Investigate why the Logs screen still feels substantially slower than expected after task-226 eliminated idle busy rendering and introduced viewport projection. This is a measurement-first research task, not an implementation task.

## Outcome

The cause was found by reading the Logs code path against the real 257-document workspace. No benchmark harness was built.

### F1 — Empty filter is cheap, active filter is not

`filter_logs` (`tandem/src/tui/logs.rs:79`) short-circuits an empty query to `logs.iter().collect()`, which is 257 pointer copies and effectively free. Any non-empty query falls through to `log_matches_query` for every log.

### F2 — `log_matches_query` allocates roughly twice the corpus per pass

For each document it builds a fresh `String`, pushes the entire `doc.body` into it, then calls `.to_ascii_lowercase()`, allocating a second full copy. `.tandem/logs` is 1.3 MB across 257 documents, so a single filter pass allocates and copies about 2.5 MB. It also re-lowercases the query inside the per-document loop (`logs.rs:131`) although the caller already lowercased it at `logs.rs:79`.

### F3 — Each pass is recomputed several times per keystroke

`filtered_logs()` has 8 call sites and caches nothing:

| Call site | Purpose |
| --- | --- |
| `state.rs:751,764,772` | navigation bounds |
| `state.rs:888` | `clamp_selection` log count |
| `state.rs:995` | `selected_log` builds the whole Vec to take one element |
| `state.rs:999` | `select_log_by_id_preserving_scroll` |
| `state.rs:1029` | `logs_status_message` |
| `state.rs:1131` | `draw_log_list` |

`log_detail_line_count` (`state.rs:1023`) calls `selected_log()` and then constructs the entire detail-line vector only to read `.len()`. Typing one character in the search box runs roughly five filter passes and two full detail-line builds, so a five-character query does the work about 25 times.

### F4 — The prior benchmark could not observe this

task-226's harness measured idle CPU and selection redraw, which exercise the empty-filter fast path in F1. That is why it reported approximately 55 ms while real use drags. Reconstructing it would have reproduced the same blind spot.

### Measurement caveat

Timings are CLI proxies on the real workspace: `tandem log list` 41 ms, `tandem log search` 114 ms, so roughly 73 ms attributable to one filter pass. CLI search uses `app::queries::search_match`, not the TUI's `log_matches_query`, so treat 73 ms as an order-of-magnitude signal rather than the TUI's exact figure. The per-keystroke multiplication in F3 is read from call sites, not measured.

## Benchmark harness decision

No harness is needed and none was built. `scripts/benchmark_tui_idle.py` was removed in task-242 and is not being replaced. No PTY dependency is added, so no stack decision is required. If a future performance question genuinely needs a harness, it should be justified by that question rather than inherited from this one.

## Follow-up

Implementation is tracked separately; this task delivered diagnosis only.
