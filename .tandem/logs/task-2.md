---
id: task-2
type: task
title: "Implement first TUI board mutation flow"
priority: "high"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/src/main.rs"]
tags: ["tui", "mutation", "mvp"]
createdAt: "2026-06-27T03:58:59Z"
updatedAt: "2026-06-27T04:35:47Z"
subtasks:
  - id: task-2-1
    title: "Choose the smallest move/change-state UX"
    completed: false
  - id: task-2-2
    title: "Reuse or extract existing move mutation logic"
    completed: false
  - id: task-2-3
    title: "Reload board and surface write errors safely"
    completed: false
accord:
  status: "accepted"
  assignee: "herd:tui-board-mutation"
  claimedAt: "2026-06-27T04:04:32Z"
  deliveredAt: "2026-06-27T04:12:05Z"
  deliverables: ["code:tandem-tui/src/tui.rs:first board mutation flow", "code:tandem-tui/src/main.rs:shared mutation helper if needed"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Keep CLI behavior unchanged except for shared helper extraction needed by the TUI.", "Do not add a root workspace, schemas, fixtures, or Brainfile migration work."]
  summary: "Implemented first TUI board mutation: H/L moves the selected task between configured states, reloads the board, and reports move errors in the status line."
  evidence: ["cargo fmt --check, cargo test, cargo build, git diff --check passed; PTY smoke moved a temp task todo -> in-progress -> todo through tdm tui."]
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  reviewer: "orchestrator"
  note: "Verified by orchestrator: cargo fmt/test/build, git diff --check, and PTY smoke passed."
  updatedAt: "2026-06-27T04:35:47Z"
completedAt: "2026-06-27T04:35:47Z"
completion:
  summary: "Implemented first TUI board mutation: H/L moves selected tasks between configured states, reloads, and surfaces errors in the status line."
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  validation: "cargo fmt --check, cargo test, cargo build, git diff --check, and PTY smoke passed"
  reviewer: "orchestrator"
---

## Description

Add the first in-TUI mutation, preferably moving the selected task between states, with safe write errors and reload after success.
