---
id: task-8
type: task
title: "Implement TUI theme loading and visual language"
state: todo
priority: "medium"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/Cargo.toml", ".tandem/theme.toml"]
tags: ["tui", "theme", "mvp"]
createdAt: "2026-06-27T03:59:00Z"
updatedAt: "2026-06-27T03:59:00Z"
subtasks:
  - id: task-8-1
    title: "Define exact TOML theme keys"
    completed: false
  - id: task-8-2
    title: "Apply built-in defaults and workspace override"
    completed: false
  - id: task-8-3
    title: "Define accord/review/priority badge styles"
    completed: false
accord:
  status: "ready"
  deliverables: ["code:tandem-tui/src/tui.rs:theme loading and styling", "docs:tandem-tui/plan/spec.md:theme key details if behavior changes"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Stop and report before adding a new dependency beyond the current minimal stack unless clearly justified."]
  updatedAt: "2026-06-27T03:59:00Z"
---

## Description

Load/apply built-in, user, and workspace themes and settle visible badge/status styling for priority, accord, review, and selection.
