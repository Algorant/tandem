---
id: task-9
type: task
title: "Harden TUI runtime interactions and rendering"
priority: "medium"
relatedFiles: ["tandem/src/tui.rs", "tandem/plan/spec.md"]
tags: ["tui", "keyboard", "mouse", "markdown"]
createdAt: "2026-06-27T03:59:00Z"
updatedAt: "2026-06-28T17:30:32Z"
subtasks:
  - id: task-9-1
    title: "Implement hot reload and safe external edit/error handling"
    completed: true
  - id: task-9-2
    title: "Render styled-basic Markdown in detail panes"
    completed: true
  - id: task-9-3
    title: "Improve mouse hit-map and action-button interactions"
    completed: false
  - id: task-9-4
    title: "Finalize keybinding/help table discoverability"
    completed: false
accord:
  status: "claimed"
  assignee: "pi"
  claimedAt: "2026-06-28T02:27:57Z"
  deliverables: ["code:tandem/src/tui.rs:runtime interaction polish", "docs:tandem/plan/spec.md:keybinding/mouse/markdown details if behavior changes"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test", "cd tandem && cargo build"]
  constraints: ["Keep keyboard-first behavior; drag/drop stays out of v0."]
  updatedAt: "2026-06-28T03:32:31Z"
completedAt: "2026-06-28T17:30:32Z"
completion:
  summary: "Rescoped broad TUI runtime tracker: hot reload and Markdown rendering are complete; remaining mouse/action-button and keybinding/help work is split into task-42 and task-43."
---

## Description

Finish MVP interaction polish as a parent task tracked through focused subtasks.

Context from orchestration:
- This task is intentionally broad and should not be delegated as one monolithic implementation slice.
- Use the subtasks below to keep work reviewable and reduce conflicts in `tandem/src/tui.rs`.
- Delegate hot reload/error handling and Markdown rendering first; mouse/action-button work should follow once runtime stability is clearer; final keybinding/help cleanup should come last.

Subtask direction:
1. `task-9-1` — hot reload plus safe external edit handling: reload board/config/log/theme files, preserve selection where possible, and surface parse/write errors without crashing.
2. `task-9-2` — styled-basic Markdown rendering: improve headings, lists, code fences, blockquotes, and related detail/body rendering without adding dependencies unless clearly justified.
3. `task-9-3` — mouse hit-map/action buttons: formalize clickable regions and add button clicks where actions are already hinted; keep drag/drop out of v0.
4. `task-9-4` — final keybinding/help table: reconcile help/footer/status text after the behavior slices land.

Acceptance direction:
- Keep each subtask independently reviewable with focused tests where practical.
- Avoid broad docs churn; update TUI docs only for user-visible behavior changes.
- Keep keyboard-first behavior and existing Board/Review/Logs/Rules/Decisions flows intact.
