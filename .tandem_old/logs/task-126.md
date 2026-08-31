---
id: task-126
type: task
title: "Implement hierarchical subtask ID allocation in Tandem CLI"
priority: "high"
parentId: "task-101"
blockers: ["task-125"]
references: ["decision-4", "task-103"]
relatedFiles: ["tandem/src/main.rs", "tandem/README.md", "tandem/plan/spec.md"]
tags: ["tui", "cli", "subtasks", "ids"]
createdAt: "2026-07-14T00:54:30Z"
updatedAt: "2026-07-14T02:27:41Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-14T02:17:51Z"
  deliveredAt: "2026-07-14T02:25:22Z"
  deliverables: ["Focused commit 71a907f7faa8ea28246d0dca528e7a61d9b777d9 on shep/task-126-implement-hierarchical-subtask-id-alloca", "Hierarchical and nested task-child allocation with collision-safe creation", "CLI output/reparenting compatibility plus tests and docs"]
  validation:
    commands: ["Worker: cargo fmt --check passed", "Worker: cargo test passed, 129 tests", "Worker: Bun pi-tandem relationship smoke passed", "Worker: manual CLI smoke for JSON, completed-log non-reuse, nesting, and generic parents passed", "Worker: git diff --check passed; clippy blocked by reported pre-existing unrelated warnings"]
  summary: "PASS. Parent inspected commit 71a907f7faa8ea28246d0dca528e7a61d9b777d9 and the allocation/reparenting/JSON/test implementation, independently validated hierarchical, nested, completed-log, generic-parent, legacy-flat, list/search/show, completion, and immutable-reparent behavior, confirmed atomic concurrent allocation, and fast-forwarded it to main."
  evidence: ["Clean worktree and exactly three intended changed files", "Worktree /home/ivan/.pi/agent/worktrees/tandem/task-126-implement-hierarchical-subtask-id-alloca", "No unexpected files or blockers; READY FOR PARENT DELIVERY"]
  filesChanged: ["tandem/src/main.rs", "tandem/README.md", "tandem/plan/spec.md"]
  reviewer: "pi"
  updatedAt: "2026-07-14T02:27:33Z"
completedAt: "2026-07-14T02:27:41Z"
completion:
  summary: "Implemented hierarchical first-class subtask allocation in the Tandem CLI: task parents allocate parent-derived and nested IDs, generic parents retain flat IDs, completed IDs are not reused, concurrent creation is atomic, IDs remain immutable on reparenting, JSON/human output exposes relationships, and compatibility behavior is tested and documented."
  filesChanged: ["tandem/src/main.rs", "tandem/README.md", "tandem/plan/spec.md"]
  validation: "PASS. Parent reviewed and integrated commit 71a907f7faa8ea28246d0dca528e7a61d9b777d9; ran cargo fmt --check, all 129 Rust tests, Bun pi-tandem relationship smoke, git diff --check, clippy triage, and an independent CLI smoke covering JSON creation, hierarchical/nested IDs, completion/log non-reuse, generic parents, immutable reparent warnings, and show/list/search compatibility."
  reviewer: "pi"
---

## Description

Implement decision-4 after the protocol correction.

Acceptance criteria:
- `tandem add --parent task-103` resolves the parent and, when it is a task, allocates `task-103-1`, then `task-103-2`, etc.
- Nested allocation extends the full parent ID, such as `task-103-1-1`.
- Parent links to non-task documents remain valid generic relationships and use normal flat task allocation rather than subtask designation.
- Allocation scans board and completed logs, never reuses IDs, and fails safely on collisions/concurrent changes without overwriting.
- `parentId` is always persisted and remains canonical.
- Existing flat-ID children remain readable, searchable, filterable, and completable.
- IDs remain immutable; `update --parent` must preserve the ID and clearly handle/warn about designation mismatch rather than silently renaming references.
- Human/JSON add output, show/list/search behavior, validation, and tests cover hierarchical, nested, legacy-flat, generic-parent, collision, completion/log, and reparent cases.
- Do not restore legacy inline checklist authoring.
