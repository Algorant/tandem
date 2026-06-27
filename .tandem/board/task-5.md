---
id: task-5
type: task
title: "Implement Review queue view"
state: todo
priority: "high"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/plan/spec.md"]
tags: ["tui", "review", "accord", "mvp"]
createdAt: "2026-06-27T03:58:59Z"
updatedAt: "2026-06-27T03:59:00Z"
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
  status: "ready"
  deliverables: ["code:tandem-tui/src/tui.rs:Review queue view"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Keep v0 Review queue as a filtered list, not hard-coded workflow sections."]
  updatedAt: "2026-06-27T03:59:00Z"
---

## Description

Create the simple filtered Review queue for delivered accords, pending review, blocked work, and accepted-but-not-completed tasks.
