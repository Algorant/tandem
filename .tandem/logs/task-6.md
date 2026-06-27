---
id: task-6
type: task
title: "Implement Logs view list/show/search"
priority: "medium"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/src/main.rs"]
tags: ["tui", "logs", "mvp"]
createdAt: "2026-06-27T03:59:00Z"
updatedAt: "2026-06-27T16:56:48Z"
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
  status: "accepted"
  assignee: "herd:tui-logs-view"
  claimedAt: "2026-06-27T14:48:05Z"
  deliveredAt: "2026-06-27T15:04:57Z"
  deliverables: ["code:tandem-tui/src/tui.rs:Logs view"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Restore/reopen remains deferred unless explicitly requested."]
  summary: "Implemented the TUI Logs view with recency-sorted completed-log list, detail pane, / search filter, safe load warnings, and event context while preserving Board quick-add/move behavior."
  evidence: ["Worktree commit ce63f421dee2bab09150831d4639048b2cf4a88a. Validation passed: cd tandem-tui && cargo fmt --check; cargo test; cargo build; git diff --check. PTY smoke passed in a temp workspace: completed a log, opened Logs, searched/navigated detail, returned to Board, quick-added a task, moved it with L, and verified list/log search output."]
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/logs.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  reviewer: "orchestrator"
  note: "Reviewed and integrated Logs branch; validation and PTY smoke passed."
  updatedAt: "2026-06-27T16:56:48Z"
completedAt: "2026-06-27T16:56:48Z"
completion:
  summary: "Integrated Logs TUI pane with recency list, detail, search, events, validation, and smoke coverage."
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/logs.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  validation: "cargo fmt --check && cargo test && cargo build; git diff --check; PTY Logs smoke"
  reviewer: "orchestrator"
---

## Description

Make completed logs useful in the TUI with list, detail, and basic search over .tandem/logs plus event context where available.
