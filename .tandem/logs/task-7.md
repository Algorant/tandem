---
id: task-7
type: task
title: "Implement Rules and Decisions TUI views"
priority: "medium"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/src/main.rs"]
tags: ["tui", "rules", "decisions", "mvp"]
createdAt: "2026-06-27T03:59:00Z"
updatedAt: "2026-06-27T22:45:02Z"
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
  status: "accepted"
  assignee: "herd:tui-rules-decisions"
  claimedAt: "2026-06-27T14:48:05Z"
  deliveredAt: "2026-06-27T19:11:55Z"
  deliverables: ["code:tandem-tui/src/tui.rs:Rules and Decisions views"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Stay aligned with existing CLI v0 behavior; protocol semantics should not change."]
  summary: "Rebased task-7 Rules/Decisions TUI work onto current main with Review and Logs panes preserved"
  evidence: ["Branch head da408ef based on 820cc82. Validation passed: cargo fmt --check, cargo test (32 passed), cargo build, git diff --check main..HEAD. PTY smoke covered Rules add/edit/delete and Decisions browse/add after rebase."]
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/rules.rs", "tandem-tui/src/tui/decisions.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  reviewer: "orchestrator"
  note: "Reviewed rebased Rules/Decisions integration; final validation and integrated PTY smoke passed."
  updatedAt: "2026-06-27T22:45:01Z"
completedAt: "2026-06-27T22:45:02Z"
completion:
  summary: "Integrated Rules and Decisions TUI panes with separate source modules, validation, and smoke coverage."
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/rules.rs", "tandem-tui/src/tui/decisions.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  validation: "cargo fmt --check && cargo test && cargo build; git diff --check; integrated Review/Logs/Rules/Decisions PTY smoke"
  reviewer: "orchestrator"
---

## Description

Add TUI surfaces for project rules and decision documents, matching the existing v0 CLI capabilities.
