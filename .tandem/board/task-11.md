---
id: task-11
type: task
title: "Adopt Brainfile-style Board state subviews"
state: todo
priority: "high"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/theme.rs", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
tags: ["tui", "board", "layout", "brainfile-inspired", "subviews"]
createdAt: "2026-06-27T15:02:47Z"
updatedAt: "2026-06-27T15:02:47Z"
subtasks:
  - id: task-11-1
    title: "Replace multi-column Board layout with selected-state subview list"
    completed: false
  - id: task-11-2
    title: "Design richer Brainfile-inspired task rows"
    completed: false
  - id: task-11-3
    title: "Preserve quick-add, move, detail, mouse, and theme behavior"
    completed: false
  - id: task-11-4
    title: "Consider reusable subview model for other top-level panes"
    completed: false
accord:
  status: "ready"
  deliverables: ["code:tandem-tui/src/tui.rs:single-state Board subview layout", "docs:tandem-tui/plan/spec.md:Board subview layout direction"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Borrow the interaction model, not Brainfile terminology; keep Tandem state/accord/review vocabulary.", "Do not add a persistent done column."]
  updatedAt: "2026-06-27T15:02:47Z"
---

## Description

Rework the Board view from simultaneous columns into a Brainfile-style single-state subview list with richer row information.

User feedback and screenshot observations:
- Current Tandem Board shows todo, in-progress, and review columns at the same time; the columns feel too narrow and limit useful at-a-glance information.
- Brainfile's Board/Tasks UI shows one task state/submenu at a time, with state sub-tabs such as `TO DO 2` and `IN PROGRESS 1`; the active subview is highlighted and gets the full content width.
- Brainfile rows are dense and readable: priority badge, title, accord/contract status and checklist progress inline, tags/metadata on a second line, and right-aligned task id.
- Footer/status reinforces the active subview count such as `TO DO 1/2` or `IN PROGRESS 1/1`.
- This subview model likely generalizes well: other top-level panes (Review, Logs, Rules, Decisions) can also have their own subviews/filters without cramming multiple panes on screen.

Acceptance direction:
- Keep existing top-level views (`1` Board, `2` Review, `3` Logs, `4` Rules, `5` Decisions), but make Board itself use selected state subviews instead of multi-column layout.
- Board state tabs should show configured states and counts, with the active state highlighted.
- Use the full Board content area for the selected state list, making rows richer and more Brainfile-like.
- Preserve or adapt existing controls: quick-add `a`, move selected task `H`/`L`, state/subview switching, detail pane, mouse tab/state selection, and theme styling.
- Consider a reusable view/subview abstraction so Review/Logs/Rules/Decisions can grow subviews later.
- Do not reintroduce a persistent done column; completion remains archive-to-logs.
