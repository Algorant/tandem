---
id: task-36
type: task
title: "Align TUI Board and Logs ordering with protocol 0.3.0 timestamps"
state: "in-progress"
priority: "medium"
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/chrome.rs", "tandem/src/web.rs"]
tags: ["tui"]
accord:
  status: "claimed"
  acceptance: ["Logs list is newest-first using protocol 0.3.0 archive time (`archivedAt`), not stripped `completedAt`. Historical logs that only have `completedAt` still sort instead of collapsing to ID order.", "Log chrome and detail show archive recency from `archivedAt` (or a real fallback), never `completed unknown` when `archivedAt` is present.", "Board task lists are newest-first within workflow columns (createdAt, then numeric ID). No user-facing sort UI.", "Leftover `completedAt` recency sorts that would show sequential IDs on 0.3.0 archives (TUI Logs and the matching web logs list) use `archivedAt` with the same fallback.", "Tests cover newest-first Logs and within-column Board order, including archives that have `archivedAt` and no `completedAt`."]
  claimedAt: "2026-09-17T02:16:08Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml --lib"]
  constraints: ["Do not reintroduce required `completedAt` or rewrite archived log files.", "Do not add sort controls, keymap, or config.", "Do not change protocol semantics, CLI list order, or Decisions ID order unless required to share a recency helper.", "Keep Board column grouping by workflow state; recency is within-column only."]
  updatedAt: "2026-09-17T02:16:08Z"
createdAt: "2026-09-17T02:08:57Z"
updatedAt: "2026-09-17T02:16:08Z"
effort: "medium"
assignee: "worker-task-36-d8ce97ef"
---
Protocol 0.3.0 cleanup, not a product-review spike.

Algorant wants most recent work at the top. 0.13.3 TUI Logs showed sequential IDs (`task-1` through `task-35`) and `completed unknown` because `sort_logs_by_recency` and chrome/detail still key on `completedAt`. Complete/cancel now write `archivedAt`, strip `completedAt`, and put outcome on `resolution`. This workspace has 0/34 logs with `completedAt`. Empty `completedAt` ties, so the list falls back to numeric ID order. Logs used to be newest-first; this is the switch.

Board `sort_documents` is `state` then `compare_ids` (oldest/lowest ID first). There is no sort UI; do not add one.

Required code tighten-up:
- Logs: newest-first by `archivedAt`, fallback `completedAt` then numeric ID so historical and 0.3.0 archives both work.
- Display archive time from `archivedAt` (fallback `completedAt`), not `unknown` when `archivedAt` exists.
- Board: keep state/column grouping; within a column newest-first by `createdAt` then numeric ID.
- Fix the same `completedAt` recency key in the web logs list if it is still present.
- Prefer a shared recency helper over duplicated empty-string sorts.

Do not migrate log files, do not restore `completedAt` as a required field, do not change CLI `tandem list` or Decisions ordering unless a shared helper is the smaller fix.