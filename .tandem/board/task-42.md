---
id: task-42
type: task
title: "Implement TUI mouse hit-map and action buttons"
state: todo
priority: "medium"
parentId: "task-9"
relatedFiles: ["tandem/src/tui.rs", "tandem/plan/spec.md"]
tags: ["tui", "mouse", "ux"]
createdAt: "2026-06-28T17:29:46Z"
updatedAt: "2026-06-28T17:29:46Z"
accord:
  status: "ready"
  deliverables: ["code:tandem/src/tui.rs:mouse hit-map and action-button interactions", "docs:tandem/plan/spec.md:TUI mouse behavior if user-visible semantics change"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test", "cd tandem && cargo build"]
  constraints: ["Keep keyboard-first behavior.", "Drag/drop stays out of v0.", "Reuse existing mutation/action paths for mouse-triggered actions."]
  updatedAt: "2026-06-28T17:29:46Z"
---

## Description

Follow-up split from task-9.

Scope:
- Formalize a TUI hit-map for clickable regions in the current Ratatui/crossterm Board shell.
- Add mouse clicks for action buttons that are already visible or hinted by keyboard actions.
- Cover tab/state selection, row selection, scroll, detail/inline expansion, and safe no-op behavior for non-action regions.
- Keep drag/drop out of v0 and preserve keyboard-first behavior.

Traceability:
- Parent tracker: task-9.
- Replaces open subtask task-9-3.

Acceptance:
- Mouse interactions reuse existing mutation/action paths rather than creating separate behavior.
- Add focused tests or smoke coverage where practical.
- Update Tandem TUI docs/spec only for user-visible behavior changes.
