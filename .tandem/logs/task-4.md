---
id: task-4
type: task
title: "Add top-level TUI view switching"
priority: "high"
relatedFiles: ["tandem-tui/src/tui.rs"]
tags: ["tui", "views", "mvp"]
createdAt: "2026-06-27T03:58:59Z"
updatedAt: "2026-06-27T05:49:31Z"
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
  status: "accepted"
  assignee: "herd:tui-view-shell"
  claimedAt: "2026-06-27T04:58:34Z"
  deliveredAt: "2026-06-27T05:11:42Z"
  deliverables: ["code:tandem-tui/src/tui.rs:top-level view navigation"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Do not implement every view action in this slice; make the shell extensible."]
  summary: "Implemented top-level TUI view switching with Board, Review, Logs, Rules, and Decisions tabs, numeric keyboard switching, mouse tab hit regions, and read-only placeholder/count views while preserving Board quick-add and H/L move flows."
  evidence: ["Validation in worktree /home/ivan/dev/projects/tandem-task4-views: cargo fmt --check; cargo test (21 passed); cargo build; git diff --check; PTY smoke switched views 2/3/1, quick-added a task, moved it with L, and exited cleanly."]
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  reviewer: "orchestrator"
  note: "Verified integrated merge: cargo fmt/test/build, git diff --check, and PTY smoke with view switching plus Board quick-add/move passed."
  updatedAt: "2026-06-27T05:49:31Z"
completedAt: "2026-06-27T05:49:31Z"
completion:
  summary: "Integrated top-level TUI view switching with Board, Review, Logs, Rules, and Decisions tabs, numeric/mouse tab switching, and read-only placeholder/count views while preserving Board mutations."
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  validation: "cargo fmt --check, cargo test, cargo build, git diff --check, and integrated PTY smoke passed"
  reviewer: "orchestrator"
---

## Description

Add Board, Review, Logs, Rules, and Decisions tabs/views on the existing app shell, even if some views start as read-only placeholders.
