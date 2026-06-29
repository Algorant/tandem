---
id: task-45
type: task
title: "Add state/accord divergence warnings and tests"
state: "validation"
priority: "high"
parentId: "task-26"
blockers: ["task-30"]
relatedFiles: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/plan/spec.md"]
tags: ["cli", "tui", "accord", "state", "validation", "tests"]
createdAt: "2026-06-28T17:29:46Z"
updatedAt: "2026-06-29T00:21:24Z"
accord:
  status: "delivered"
  assignee: "tasks44-45-accord-state"
  claimedAt: "2026-06-29T00:15:41Z"
  deliveredAt: "2026-06-29T00:21:24Z"
  deliverables: ["code:tandem/src/main.rs:state/accord divergence warning surfaces for read/lint-style paths", "code:tandem/src/tui.rs:TUI detail/status divergence warning surfaces", "tests:tandem:sync behavior and divergence warning regression coverage", "docs:tandem/plan/spec.md:warning/validation candidate semantics if behavior changes"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test", "cd tandem && cargo build"]
  constraints: ["Warn about divergence without collapsing workflow state, review metadata, and accord status.", "Coordinate with task-30 for Board Validation warning and acceptance surfaces."]
  summary: "Added accord/workflow divergence warnings for read/detail paths without mutating recorded state, review metadata, or accord status. JSON list/show warnings and TUI accord detail warnings now surface sync candidates like todo+claimed and in-progress+delivered; regression tests cover warned-but-preserved divergence and synchronized movement."
  evidence: ["cd tandem && cargo fmt --check && cargo test", "Commit 6ffa20a on branch herd-task44-45-accord-state"]
  filesChanged: ["tandem/src/main.rs", "tandem/src/tui.rs"]
  updatedAt: "2026-06-29T00:21:24Z"
---

## Description

Follow-up split from task-26.

Scope:
- Surface warnings when workflow state and accord status are inconsistent, such as todo + claimed or in-progress + delivered.
- Cover CLI read/lint-style output and TUI detail/status surfaces without collapsing state, review metadata, and accord status into one concept.
- Treat delivered or verified herd work and human visual checks as validation-state candidates where conservative mapping allows it.
- Add regression tests for sync behavior and divergence warnings.

Traceability:
- Parent tracker: task-26.
- Replaces open subtasks task-26-4, task-26-5, and task-26-6.
- Coordinate with task-30 for Board Validation warning and acceptance surfaces.

Acceptance:
- Divergence warnings are visible in read/detail paths before users mutate data.
- Tests cover both synchronized transitions and intentionally warned-but-preserved divergence.
