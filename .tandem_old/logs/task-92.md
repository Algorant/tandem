---
id: task-92
type: task
title: "Investigate Tandem task ID race condition on concurrent adds"
priority: "low"
references: ["decision-2"]
relatedFiles: ["tandem/src", ".tandem/board"]
tags: ["protocol", "tasks", "concurrency", "bug", "validation"]
createdAt: "2026-07-04T13:06:20Z"
updatedAt: "2026-07-04T18:51:11Z"
accord:
  status: "accepted"
  assignee: "herd:task-92"
  claimedAt: "2026-07-04T17:59:12Z"
  deliveredAt: "2026-07-04T18:41:01Z"
  deliverables: ["Atomic create-with-retry path for new sequential documents using temp-file write plus non-replacing final-path reservation and retry on collision.", "CLI `tandem add` and `tandem decision add` now use the safe allocator.", "TUI quick task add and TUI decision add now use the safe allocator.", "Regression test `concurrent_task_adds_allocate_unique_ids_without_overwrite` verifies parallel adds produce unique task IDs without overwrites."]
  validation:
    commands: ["`cargo test --manifest-path tandem/Cargo.toml concurrent_task_adds_allocate_unique_ids_without_overwrite` passed.", "`cargo test --manifest-path tandem/Cargo.toml` passed: 117 passed.", "Manual temp-workspace 8-process concurrent `tandem add` smoke produced 8 files with IDs task-1 through task-8.", "`git diff --check -- tandem/src/main.rs tandem/src/tui.rs tandem/src/tui/decisions.rs` passed.", "`cargo build --manifest-path tandem/Cargo.toml` passed."]
  summary: "Accepted objective validated race-condition fix. Reviewed task-92 subset and validation evidence; implementation adds atomic create-with-retry for sequential task/decision document creation across CLI and TUI add paths."
  evidence: ["Race cause verified by reading ID allocation/write path: sequential ID was computed before file creation and `write_atomic` used temp write then rename to final path, which can replace an existing file.", "Task-92 implementation intentionally scoped to `tandem/src/main.rs`, `tandem/src/tui.rs`, and `tandem/src/tui/decisions.rs`; other modified files in the shared worktree are unrelated concurrent worker changes."]
  filesChanged: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/decisions.rs"]
  reviewer: "parent/orchestrator"
  updatedAt: "2026-07-04T18:50:43Z"
completedAt: "2026-07-04T18:51:11Z"
completion:
  summary: "Completed concurrent add race-condition fix. New task/decision document creation now uses atomic create-with-retry to reserve sequential IDs without replacing existing files, covering CLI task add, CLI decision add, TUI quick task add, and TUI decision add."
  validation: "Parent/orchestrator reviewed task-92 subset and reran validation: targeted concurrent add regression passed, full `cargo test --manifest-path tandem/Cargo.toml` passed with 117 tests, `cargo build --manifest-path tandem/Cargo.toml` passed, and `git diff --check` passed for task-92 files."
  reviewer: "parent/orchestrator"
---

## Description

Investigate the possible race condition observed when multiple `tandem_task action=add` operations were run in parallel: three intended task creations all reported/used `task-89`, and only one survived in the active task list. This happened while creating Bun migration tasks after decision-2.

Expected work:
- Reproduce or reason about concurrent task creation behavior.
- Inspect ID allocation and file creation/write path for non-atomic behavior.
- Propose or implement a guard such as file locking, atomic create-with-retry, or serialized add handling.
- Add validation that concurrent task adds cannot collide or overwrite each other.
- Preserve v0 simplicity and avoid broad protocol changes unless necessary.
