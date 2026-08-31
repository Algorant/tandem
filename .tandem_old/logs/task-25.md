---
id: task-25
type: task
title: "Remove top-level Review pane from TUI"
priority: "high"
relatedFiles: ["tandem/src/tui.rs", "tandem/src/tui/review.rs", "tandem/plan/spec.md", "tandem/plan/todo.md"]
tags: ["tui", "navigation", "validation", "ux"]
createdAt: "2026-06-28T04:30:03Z"
updatedAt: "2026-06-28T17:33:50Z"
subtasks:
  - id: task-25-1
    title: "Remove Review from TuiView/top-level tab labels, header counts, and navigation order"
    completed: false
  - id: task-25-2
    title: "Update keyboard/mouse top-level switching tests for the revised view set"
    completed: false
  - id: task-25-3
    title: "Remove or park unreachable Review queue rendering and selection code"
    completed: false
  - id: task-25-4
    title: "Preserve Board validation state and review/accord metadata badges/details"
    completed: false
  - id: task-25-5
    title: "Update CLI/TUI docs and todo references that list Review as a top-level pane"
    completed: false
  - id: task-25-6
    title: "Run focused TUI tests and cargo test"
    completed: false
accord:
  status: "ready"
  assignee: "pi"
  deliverables: ["code:tandem/src/tui.rs:top-level Review pane removed from TUI navigation/rendering/help/status paths", "code:tandem/src/tui/review.rs:Review queue code removed, parked, or left unreachable only if justified by tests/docs", "docs:tandem/plan/spec.md:top-level TUI view list updated to remove Review and rely on Board Validation", "tests:tandem:updated navigation/keymap tests plus cargo test evidence"]
  validation:
    commands: ["cd tandem && cargo test"]
  constraints: ["Do not remove the Board validation state/subview or default protocol state.", "Do not remove review/accord metadata parsing, CLI support, badges, or log detail fields.", "Do not introduce a replacement top-level Review/Attention pane in this task."]
  updatedAt: "2026-06-28T04:30:10Z"
completedAt: "2026-06-28T17:33:50Z"
completion:
  summary: "Removed the top-level Review pane from TUI navigation/rendering/help/status paths while preserving review metadata and Board Validation state."
  validation: "cd tandem && cargo fmt --check && cargo test && cargo build"
---

## Description

Remove the current top-level Review pane from the TUI because it muddles review metadata with workflow placement. Delivered work should be handled in the Board `validation` state/subview.

Scope:
- Remove Review from top-level tab/navigation surfaces.
- Preserve the Board `validation` state/subview as an active workflow state.
- Preserve review/accord metadata parsing, badges, and detail fields where useful.
- Do not remove protocol review metadata or CLI review/accord support.
- Park or delete unreachable read-only Review queue code/tests after navigation removal.
- Update docs/spec/todo references and tests so the v0 top-level view set no longer lists Review as a first-class pane.

Acceptance criteria:
- `tandem tui` top-level views no longer include Review.
- Numeric top-level switching and mouse tab hits match the revised view order.
- Header/status/help text no longer advertises the removed Review pane.
- Board `validation` state appears as the workflow state/subview for delivered work.
- Existing review/accord badges/details still render for Board/log documents where applicable.
- Relevant tests pass, including revised navigation/keymap coverage.
