---
id: task-36
type: task
title: "Align TUI Board and Logs ordering with protocol 0.3.0 timestamps"
priority: "medium"
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/chrome.rs", "tandem/src/web.rs"]
tags: ["tui"]
accord:
  status: "accepted"
  acceptance: ["Logs list is newest-first using protocol 0.3.0 archive time (`archivedAt`), not stripped `completedAt`. Historical logs that only have `completedAt` still sort instead of collapsing to ID order.", "Log chrome and detail show archive recency from `archivedAt` (or a real fallback), never `completed unknown` when `archivedAt` is present.", "Board task lists are newest-first within workflow columns (createdAt, then numeric ID). No user-facing sort UI.", "Leftover `completedAt` recency sorts that would show sequential IDs on 0.3.0 archives (TUI Logs and the matching web logs list) use `archivedAt` with the same fallback.", "Tests cover newest-first Logs and within-column Board order, including archives that have `archivedAt` and no `completedAt`."]
  claimedAt: "2026-09-17T02:16:08Z"
  deliveredAt: "2026-09-17T02:23:30Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml"]
  constraints: ["Do not reintroduce required `completedAt` or rewrite archived log files.", "Do not add sort controls, keymap, or config.", "Do not change protocol semantics, CLI list order, or Decisions ID order unless required to share a recency helper.", "Keep Board column grouping by workflow state; recency is within-column only."]
  summary: "Implemented protocol 0.3.0 recency ordering. Added shared helpers `archive_timestamp` (archivedAt, then legacy completedAt, blank treated as absent) and `compare_recency_desc` (first non-empty field newest-first, then canonical numeric ID; missing timestamps sort last) in protocol::workflow. TUI `sort_logs_by_recency` and web `logs_api` now sort by `[\"archivedAt\",\"completedAt\"]`; TUI and web `sort_documents` keep the `state` primary key and order within a column by `[\"createdAt\"]`. TUI Log detail, the Logs header context, and the fallback Logs panel render archive time via `archive_timestamp` instead of raw `completedAt`, so 0.3.0 archives no longer show `completed unknown`. The web log list display now reads `item.archivedAt` (the DTO field) instead of the nonexistent `item.completedAt`. Seven tests added (2 protocol, 3 TUI Logs, 1 TUI Board, 1 web logs); full suite passes. Real workspace check: 34/34 logs carry `archivedAt` and 0 carry `completedAt`, matching the reported failure. No archive migration, no required `completedAt`, no sort UI, no keymap/config, and no CLI/Decisions order changes."
  evidence: ["Logs list is newest-first using archivedAt, and historical logs with only completedAt still sort instead of collapsing to ID order.: tui/logs.rs `sort_logs_by_recency` is now `compare_recency_desc(a, b, &[\"archivedAt\", \"completedAt\"])`; test `sorts_logs_newest_first_by_archived_at_with_completed_at_fallback` asserts order [task-10(completedAt 2026-03-01), task-3(archivedAt 2026-02-01), task-1(archivedAt 2026-01-01), task-2(undated)] and `archived_at_takes_priority_over_completed_at_for_recency` asserts archivedAt wins when both are present. Real workspace: 34/34 logs have archivedAt, 0 have completedAt.", "Log chrome and detail show archive recency from archivedAt (or a real fallback), never `completed unknown` when archivedAt is present.: tui/logs.rs detail `Log reference` and tui/chrome.rs header + fallback Logs panel call `archive_timestamp(doc)`; test `log_detail_shows_archived_at_when_completed_at_is_missing` asserts the rendered detail contains `06-28 17:34` and not `unknown`. For each of the 34 real logs, archive_timestamp resolves the archivedAt value that `completed_at_compact` renders.", "Board task lists are newest-first within workflow columns (createdAt, then numeric ID), with no sort UI.: `sort_documents` in tui/mod.rs and web.rs now keeps `state` primary and adds `compare_recency_desc(a, b, &[\"createdAt\"])`; test `sort_documents_orders_within_state_newest_first_by_created_at` asserts [task-3(in-progress), task-2(03-01), task-10(03-01, ID tiebreak), task-1(01-01), task-4(no createdAt last)]. No keymap, config, or sort control was added.", "Leftover completedAt recency sorts (TUI Logs and the matching web logs list) use archivedAt with the same fallback.: web.rs `logs_api` now uses `compare_recency_desc(a, b, &[\"archivedAt\", \"completedAt\"])`; test `logs_api_orders_archived_records_newest_first_with_completed_at_fallback` asserts API order [task-3(2026-08-06 archivedAt), task-2(2026-08-05 archivedAt), task-10(2026-08-04 legacy completedAt)]. web/ui.js row now reads `item.archivedAt`, matching LogSummaryDto's serialized `archivedAt`.", "Tests cover newest-first Logs and within-column Board order, including archives with archivedAt and no completedAt.: 7 new tests were added and all pass in the 308-test run; fixtures include documents with archivedAt only, completedAt only, both, and neither. No protocol semantics, CLI list order, or Decisions order were changed (web.rs still uses `compare_ids` for decisions)."]
  filesChanged: ["tandem/src/protocol/workflow.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/mod.rs", "tandem/src/web.rs", "tandem/src/web/ui.js"]
  updatedAt: "2026-09-17T02:23:30Z"
createdAt: "2026-09-17T02:08:57Z"
updatedAt: "2026-09-17T02:23:30Z"
effort: "medium"
assignee: "worker-task-36-d8ce97ef"
archivedAt: "2026-09-17T02:23:30Z"
resolution:
  outcome: "completed"
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