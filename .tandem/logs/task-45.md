---
id: task-45
type: task
title: "Board: cycleable sort modes and collapsed-by-default hierarchy"
priority: "medium"
references: ["task-36"]
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/state.rs", "tandem/src/tui/bindings.rs", "tandem/src/tui/chrome.rs"]
tags: ["tui", "keyboard"]
accord:
  status: "accepted"
  acceptance: ["`s` on the Board cycles ID ascending → ID descending → newest created → recently updated → priority → back; the default on launch is ID ascending and the choice is session-only.", "The active sort applies at every hierarchy level (roots, Epic children, Subtasks) in State Board and Epic Board, with state grouping preserved; the active mode is visible in Board chrome.", "No Board row is auto-expanded by state mismatch or by active filters; Enter/click reliably toggles open and closed, and the status message matches the resulting state.", "Collapsed rows hiding pane-matching or filter-matching descendants show a hint count.", "The key reference/help lists `s`; tests cover each sort mode at child level and collapse behavior for cross-state parents with and without filters; `just dev-check` passes and the rendered Board is verified in a Herdr pane."]
  claimedAt: "2026-09-26T19:24:15Z"
  deliveredAt: "2026-09-26T19:35:10Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml", "$ just dev-check"]
  summary: "Revalidated unchanged delivery against corrected Task scope: five session-only Board sort modes on s and explicit-only State Board hierarchy expansion with hidden-match hints. Epic Board retains Enter inline preview."
  evidence: ["Sort cycling and uniform child-level ordering: BoardSort defaults ID ascending, cycles five modes and wraps; tests exercise numeric IDs and each mode on Epic children and Subtasks in both arrangements. Reload retains session sort and key reference includes s; cycling preserves selected document ID.", "Collapsed hierarchy and accurate hints: Projection tests verify an in-progress Epic with todo child remains collapsed in todo pane with and without tag filter, displays one hidden match, and reveals child only on explicit expansion; Herdr pane verified rendered collapse and Enter open/close.", "Epic Board semantics: Epic Board projection always displays descendant rows and Enter continues to toggle inline preview; no equivalent auto-expansion was present to remove."]
  filesChanged: ["tandem/src/tui/bindings.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/input.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/state.rs"]
  updatedAt: "2026-09-26T19:35:17Z"
createdAt: "2026-09-26T19:23:00Z"
updatedAt: "2026-09-26T19:35:17Z"
effort: "medium"
assignee: "worker-task-45-d8de4d68"
archivedAt: "2026-09-26T19:35:17Z"
resolution:
  outcome: "completed"
---

## Description

Requested by Algorant. Two TUI Board changes, tracked as Subtasks.

## 1. Sort modes (supersedes task-36's "no sort UI" constraint)
Today `tui/mod.rs::sort_documents` orders by `state` then `createdAt` newest-first (from task-36), and the tree walk in `board/mod.rs` uses that order. So Epics, Tasks under an Epic, and Subtasks all show newest-first. Algorant does not always want that.

Add an unbound `s` key that cycles sort modes directly, with no picker:
- ID ascending (default)
- ID descending
- Newest created (`createdAt` desc; today's behavior)
- Recently updated (`updatedAt` desc)
- Priority (high to low, then ID ascending)

Session-only, no config persistence. Applies uniformly at every level (roots, Epic children, Subtasks) in both State Board and Epic Board, and keeps the grouping by workflow state. Show the active mode in Board chrome and the status line.

## 2. Collapsed by default, including with filters
`state_board_entries_with_hierarchy` / `collect_visible_state_descendants` treat a row as expanded when `expanded_ids` contains it OR (a descendant matches the pane state AND the row's own state differs). Example: on the todo pane, an in-progress/validation Epic or Task that has todo children is forced open. Enter (`toggle_board_expansion`) only changes `expanded_ids`, so the row can't be collapsed. The status line also says "Expanded… press Enter to collapse" while nothing changes. Active filters (`filters.is_active()`) also force every branch open.

Wanted: every row with children starts collapsed and opens only by explicit Enter/click, with or without filters. A collapsed row that hides pane-matching or filter-matching descendants shows a compact hint count so matches can still be found.
