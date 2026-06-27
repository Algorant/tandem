---
id: task-5
type: task
title: "Implement Review queue view"
priority: "high"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/plan/spec.md"]
tags: ["tui", "review", "accord", "mvp"]
createdAt: "2026-06-27T03:58:59Z"
updatedAt: "2026-06-27T16:56:48Z"
subtasks:
  - id: task-5-1
    title: "Define Review filters from current document metadata"
    completed: false
  - id: task-5-2
    title: "Render compact queue plus detail pane"
    completed: false
  - id: task-5-3
    title: "Show accord/review/action hints clearly"
    completed: false
accord:
  status: "accepted"
  assignee: "herd:tui-review-view"
  claimedAt: "2026-06-27T14:48:05Z"
  deliveredAt: "2026-06-27T15:12:04Z"
  deliverables: ["code:tandem-tui/src/tui.rs:Review queue view"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Keep v0 Review queue as a filtered list, not hard-coded workflow sections."]
  summary: "Implemented read-only TUI Review queue with filtered attention list, inspection detail, reason badges, navigation, docs updates, and preserved Board actions."
  evidence: ["Validation in worktree: cd tandem-tui && cargo fmt --check && cargo test && cargo build; git diff --check; PTY smoke switched to Review, navigated detail, returned to Board, quick-added and moved a task."]
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/review.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  reviewer: "orchestrator"
  note: "Reviewed and integrated Review queue branch; validation and PTY smoke passed."
  updatedAt: "2026-06-27T16:56:48Z"
completedAt: "2026-06-27T16:56:48Z"
completion:
  summary: "Integrated Review queue TUI pane with validation and smoke coverage."
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/review.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  validation: "cargo fmt --check && cargo test && cargo build; git diff --check; PTY Review smoke"
  reviewer: "orchestrator"
---

## Description

Create the simple filtered Review queue for delivered accords, pending review, blocked work, and accepted-but-not-completed tasks.
