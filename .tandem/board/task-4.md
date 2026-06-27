---
id: task-4
type: task
title: "Add top-level TUI view switching"
state: todo
priority: "high"
relatedFiles: ["tandem-tui/src/tui.rs"]
tags: ["tui", "views", "mvp"]
createdAt: "2026-06-27T03:58:59Z"
updatedAt: "2026-06-27T03:58:59Z"
subtasks:
  - id: task-4-1
    title: "Render top-level tabs"
    completed: false
  - id: task-4-2
    title: "Preserve per-view selection state"
    completed: false
  - id: task-4-3
    title: "Support keyboard and mouse tab switching"
    completed: false
accord:
  status: "ready"
  deliverables: ["code:tandem-tui/src/tui.rs:top-level view navigation"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Do not implement every view action in this slice; make the shell extensible."]
  updatedAt: "2026-06-27T03:58:59Z"
---

## Description

Add Board, Review, Logs, Rules, and Decisions tabs/views on the existing app shell, even if some views start as read-only placeholders.
