---
id: task-9
type: task
title: "Harden TUI runtime interactions and rendering"
state: todo
priority: "medium"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/plan/spec.md"]
tags: ["tui", "polish", "mouse", "markdown", "mvp"]
createdAt: "2026-06-27T03:59:00Z"
updatedAt: "2026-06-27T03:59:00Z"
subtasks:
  - id: task-9-1
    title: "Finalize fixed v0 keybinding table"
    completed: false
  - id: task-9-2
    title: "Improve mouse hit-map interactions"
    completed: false
  - id: task-9-3
    title: "Render styled-basic Markdown"
    completed: false
  - id: task-9-4
    title: "Surface reload/parse/write errors without crashing"
    completed: false
accord:
  status: "ready"
  deliverables: ["code:tandem-tui/src/tui.rs:runtime interaction polish", "docs:tandem-tui/plan/spec.md:keybinding/mouse/markdown details if behavior changes"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Keep keyboard-first behavior; drag/drop stays out of v0."]
  updatedAt: "2026-06-27T03:59:00Z"
---

## Description

Finish MVP interaction polish: fixed keybinding table, stronger mouse hit-map behavior, styled-basic Markdown rendering, file reload, and safe parse/write errors.
