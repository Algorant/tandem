---
id: task-103
type: task
title: "Adapt CLI terminology and reads for subtask hierarchy"
priority: "medium"
parentId: "task-101"
relatedFiles: ["tandem/src/main.rs", "tandem/README.md", "tandem/plan/spec.md"]
tags: ["tui", "subtasks", "cli"]
createdAt: "2026-07-05T16:22:34Z"
updatedAt: "2026-07-10T19:31:22Z"
accord:
  status: "accepted"
  assignee: "shep-task-103"
  claimedAt: "2026-07-10T17:26:18Z"
  deliveredAt: "2026-07-10T19:30:59Z"
  deliverables: ["Focused rebased commit 05ef3aa8f37aabe19e8a7b377911a3e893df3e76 on shep/task-103-adapt-cli-terminology-and-reads-for-subt", "CLI parent filters and relationship-aware human/JSON output across add, update, show, list, and search", "Regression coverage distinguishing task-parent subtasks from non-task generic parent relationships", "Updated tandem/README.md and tandem/plan/spec.md"]
  validation:
    commands: ["Parent inspected the full amended diff and confirmed the requested protocol distinction", "cargo fmt --manifest-path tandem/Cargo.toml -- --check — passed", "cargo test --manifest-path tandem/Cargo.toml — 127 passed, 0 failed (worker and parent independently)", "Parent focused temporary-workspace CLI smoke for task parent, decision parent, show/list/search, and --subtask deprecation — passed", "git diff HEAD^ HEAD --check — passed", "merge-tree against current main — no conflict markers", "Worker git status --short — clean; no unexpected files"]
  summary: "PASS. Parent reviewed the complete rebased implementation, independently validated 127 tests and focused task/decision parent CLI behavior, and fast-forwarded commit 05ef3aa to main. The objective non-visual acceptance criteria are met."
  evidence: ["Branch: shep/task-103-adapt-cli-terminology-and-reads-for-subt", "Worktree: /home/ivan/.pi/agent/worktrees/tandem/task-103-adapt-cli-terminology-and-reads-for-subt", "Commit: 05ef3aa8f37aabe19e8a7b377911a3e893df3e76", "Known dependency: pi-tandem still sends deprecated --subtask arguments; migration is tracked by task-105 and intentionally excluded here"]
  filesChanged: ["tandem/src/main.rs", "tandem/README.md", "tandem/plan/spec.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-10T19:31:15Z"
completedAt: "2026-07-10T19:31:22Z"
completion:
  summary: "Adapted Tandem CLI hierarchy behavior so parent-linked task children are first-class subtasks, non-task parent relationships remain generic, add/update/show/list/search expose the distinction, and inline checklist authoring is deprecated without hierarchical ID auto-generation."
  filesChanged: ["tandem/src/main.rs", "tandem/README.md", "tandem/plan/spec.md"]
  validation: "Parent reviewed and integrated commit 05ef3aa8f37aabe19e8a7b377911a3e893df3e76. Cargo formatting passed; all 127 Rust tests passed; focused temporary-workspace CLI smoke covering task and decision parents passed; diff, status, and merge checks passed."
  reviewer: "parent-orchestrator"
---

## Description

Adapt CLI behavior/spec around subtask hierarchy without auto-generating hierarchical IDs.

Scope:
- Ensure add/update/show/list/search surfaces clearly support parent-linked child tasks as subtasks.
- Do not auto-generate `task-100-1` IDs in v0.
- Prefer terminology that makes parent/child/subtask relationships understandable.
- Revisit or deprecate inline `subtasks` authoring paths where they conflict with the new model.
- Add validation/tests where CLI output or protocol mutation behavior changes.
