---
id: task-6
type: task
title: "Implement Logs view list/show/search"
state: todo
priority: "medium"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/src/main.rs"]
tags: ["tui", "logs", "mvp"]
createdAt: "2026-06-27T03:59:00Z"
updatedAt: "2026-06-27T03:59:00Z"
subtasks:
  - id: task-6-1
    title: "List completed logs by recency"
    completed: false
  - id: task-6-2
    title: "Show completion summary, files, validation, accord, and body"
    completed: false
  - id: task-6-3
    title: "Add basic log search/filter flow"
    completed: false
accord:
  status: "ready"
  deliverables: ["code:tandem-tui/src/tui.rs:Logs view"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Restore/reopen remains deferred unless explicitly requested."]
  updatedAt: "2026-06-27T03:59:00Z"
---

## Description

Make completed logs useful in the TUI with list, detail, and basic search over .tandem/logs plus event context where available.
