---
id: task-30
type: task
title: "Implement Board Validation flow in TUI"
priority: "high"
relatedFiles: ["tandem/src/tui.rs", "tandem/src/tui/review.rs", "tandem/plan/spec.md"]
tags: ["tui", "validation", "ux"]
blockers: ["task-25", "task-28", "task-29"]
createdAt: "2026-06-28T13:57:07Z"
updatedAt: "2026-06-28T17:33:50Z"
subtasks:
  - id: task-30-1
    title: "Render Validation as the Board state/subview for delivered work"
    completed: false
  - id: task-30-2
    title: "Ensure blocked/failed/rework attention signals do not appear in Validation solely because they need attention"
    completed: false
  - id: task-30-3
    title: "Add clear selected-task actions for approve, pass/complete, request changes, and open/edit where feasible"
    completed: false
  - id: task-30-4
    title: "Preserve accord/review badges, evidence, files changed, and human-required cues in details"
    completed: false
  - id: task-30-5
    title: "Add keyboard/mouse tests or smoke coverage for the Validation flow"
    completed: false
accord:
  status: "ready"
  assignee: "pi"
  deliverables: ["code:tandem/src/tui.rs:Board Validation state flow and actions implemented", "code:tandem/src/tui/review.rs:old top-level review queue removed, parked, or integrated only where appropriate", "docs:tandem/plan/spec.md:TUI Validation behavior documented", "tests:tandem:Validation flow coverage and cargo test evidence"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test", "cd tandem && cargo build"]
  constraints: ["Do not reintroduce a top-level Review/Attention pane in this task.", "Validation state is lifecycle placement; blocked/failed/rework remain cross-cutting signals unless actually delivered for acceptance.", "Keep implementation scoped to current Ratatui/crossterm TUI architecture."]
  updatedAt: "2026-06-28T13:57:19Z"
completedAt: "2026-06-28T17:33:50Z"
completion:
  summary: "Implemented Board Validation flow hints in the TUI for delivered work awaiting accept/rework/complete and kept blocked/failed/rework as cross-cutting signals outside Validation placement."
  validation: "cd tandem && cargo fmt --check && cargo test && cargo build"
---

## Description

Fold validation into the Board state subview instead of a top-level Review pane.

The board should present the primary flow as Todo, In Progress, Validation. Validation contains delivered work awaiting acceptance, rejection, or redirection. Broader attention signals such as blocked work should stay visible through badges/warnings, not by appearing in Validation unless the task is actually delivered for acceptance.
