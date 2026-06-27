---
id: task-7
type: task
title: "Implement Rules and Decisions TUI views"
state: todo
priority: "medium"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/src/main.rs"]
tags: ["tui", "rules", "decisions", "mvp"]
createdAt: "2026-06-27T03:59:00Z"
updatedAt: "2026-06-27T03:59:00Z"
subtasks:
  - id: task-7-1
    title: "Rules list/add/edit/delete flow"
    completed: false
  - id: task-7-2
    title: "Decisions list/show/add flow"
    completed: false
  - id: task-7-3
    title: "Preserve protocol terminology and event names"
    completed: false
accord:
  status: "ready"
  deliverables: ["code:tandem-tui/src/tui.rs:Rules and Decisions views"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Stay aligned with existing CLI v0 behavior; protocol semantics should not change."]
  updatedAt: "2026-06-27T03:59:00Z"
---

## Description

Add TUI surfaces for project rules and decision documents, matching the existing v0 CLI capabilities.
