---
id: task-8
type: task
title: "Implement TUI theme loading and visual language"
priority: "medium"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/Cargo.toml", ".tandem/theme.toml"]
tags: ["tui", "theme", "mvp"]
createdAt: "2026-06-27T03:59:00Z"
updatedAt: "2026-06-27T05:49:31Z"
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
  status: "accepted"
  assignee: "herd:tui-theme-foundation"
  claimedAt: "2026-06-27T04:58:34Z"
  deliveredAt: "2026-06-27T05:13:37Z"
  deliverables: ["code:tandem-tui/src/tui.rs:theme loading and styling", "docs:tandem-tui/plan/spec.md:theme key details if behavior changes"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Stop and report before adding a new dependency beyond the current minimal stack unless clearly justified."]
  summary: "Implemented first TUI theme foundation with default-dark palette, workspace .tandem/theme.toml overrides, no-color fallback, and themed Board styling."
  evidence: ["Validation passed in worktree: cargo fmt --check; cargo test (22 passed); cargo build; git diff --check. PTY smoke passed tdm tui with and without .tandem/theme.toml."]
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/theme.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  reviewer: "orchestrator"
  note: "Verified integrated merge: cargo fmt/test/build, git diff --check, and PTY smoke with workspace theme override passed."
  updatedAt: "2026-06-27T05:49:31Z"
completedAt: "2026-06-27T05:49:31Z"
completion:
  summary: "Integrated TUI theme foundation with default-dark palette, workspace .tandem/theme.toml overrides, no-color fallback, and themed Board/status/badge styling."
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/theme.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  validation: "cargo fmt --check, cargo test, cargo build, git diff --check, and integrated PTY smoke with theme override passed"
  reviewer: "orchestrator"
---

## Description

Load/apply built-in, user, and workspace themes and settle visible badge/status styling for priority, accord, review, and selection.
