---
id: task-3
type: task
title: "Add TUI quick-add task flow"
priority: "high"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/src/main.rs"]
tags: ["tui", "mutation", "mvp"]
createdAt: "2026-06-27T03:58:59Z"
updatedAt: "2026-06-27T04:56:59Z"
subtasks:
  - id: task-3-1
    title: "Design minimal input flow for title/state"
    completed: false
  - id: task-3-2
    title: "Create task using existing add semantics"
    completed: false
  - id: task-3-3
    title: "Handle cancel, validation, and reload"
    completed: false
accord:
  status: "accepted"
  assignee: "herd:tui-board-mutation"
  claimedAt: "2026-06-27T04:36:07Z"
  deliveredAt: "2026-06-27T04:46:09Z"
  deliverables: ["code:tandem-tui/src/tui.rs:quick-add interaction"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Keep the first flow small; no full editor or custom keymap config yet."]
  summary: "Implemented TUI quick-add: a opens a title prompt, Enter creates a basic task in the selected/default configured state, Esc cancels, and success reloads/selects the new task."
  evidence: ["cargo fmt --check, cargo test, cargo build, git diff --check passed; PTY smoke quick-added task-1 and moved it with L to verify task-2 move flow still works."]
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  reviewer: "orchestrator"
  note: "Verified by orchestrator: cargo fmt/test/build, git diff --check, and PTY quick-add/move smoke passed."
  updatedAt: "2026-06-27T04:56:59Z"
completedAt: "2026-06-27T04:56:59Z"
completion:
  summary: "Implemented TUI quick-add: a opens a title prompt, Enter creates a basic task in the selected/default configured state, Esc cancels, then reloads/selects the new task."
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  validation: "cargo fmt --check, cargo test, cargo build, git diff --check, and PTY quick-add/move smoke passed"
  reviewer: "orchestrator"
---

## Description

Let the Board view create a basic task from inside the TUI, then reload and select the new item.
