---
id: task-26
type: task
title: "Sync workflow state and accord status transitions"
state: "in-progress"
priority: "high"
relatedFiles: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/plan/spec.md"]
tags: ["tui", "cli", "accord", "state", "ux"]
createdAt: "2026-06-28T04:49:08Z"
updatedAt: "2026-06-28T15:16:45Z"
subtasks:
  - id: task-26-1
    title: "Define conservative state-to-accord and accord-to-state transition mapping"
    completed: true
  - id: task-26-2
    title: "Update CLI move and accord commands to synchronize common transitions"
    completed: true
  - id: task-26-3
    title: "Update TUI mutation paths to reuse the same synchronization behavior"
    completed: false
  - id: task-26-4
    title: "Surface divergence warnings in CLI read/lint output and TUI detail/status surfaces"
    completed: false
  - id: task-26-5
    title: "Treat delivered/verified herd work and human visual checks as validation-state candidates"
    completed: false
  - id: task-26-6
    title: "Add regression tests for sync behavior and divergence warnings"
    completed: false
accord:
  status: "claimed"
  assignee: "review-validation-flow"
  claimedAt: "2026-06-28T15:16:45Z"
  deliverables: ["code:tandem/src/main.rs:shared workflow/accord transition synchronization and warning helpers", "code:tandem/src/tui.rs:TUI mutation paths and warning surfaces use shared behavior", "docs:tandem/plan/spec.md:document state/accord synchronization semantics"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test", "cd tandem && cargo build"]
  constraints: ["Keep workflow state, review metadata, and accord status distinct; synchronize only documented common transitions.", "Avoid surprising destructive transitions; warn or prompt when mapping is ambiguous."]
  updatedAt: "2026-06-28T15:16:45Z"
---

## Description

Keep Tandem workflow state and accord status from drifting when common transition commands are used. The immediate pain point is a delegated/claimed task still appearing in `todo` instead of `in-progress`, but the fix should cover both CLI and TUI mutation paths.

Desired behavior:
- Claiming an accord should move a `todo` task to `in-progress` unless the user explicitly opts out or the task is already in a later/blocked state.
- Moving a task to `in-progress` should claim or prompt to claim its accord when an accord exists and is `ready`.
- Delivery/rework/block/fail/accept/complete transitions should have a documented, conservative state mapping instead of silently diverging.
- `/verify`-style checks of finished herd work should treat delivered work as a `validation` board-state candidate even before it is merged, committed, or put into practice.
- UI/visual work that passes automated validation but still needs a human eye check should also be eligible for `validation`; automated PASS can mean "technically ready for human/product acceptance," not necessarily complete.
- Read commands/TUI details/lint should warn when workflow state and accord status are inconsistent, such as `state: todo` with `accord.status: claimed` or `state: in-progress` with `accord.status: delivered`.

Acceptance direction:
- Use shared helpers so CLI and TUI mutations do not drift.
- Preserve the distinction between workflow state, review metadata, and accord status; synchronization should make common transitions coherent, not collapse the concepts.
- Prefer `accord.delivered -> state: validation` and `accord.claimed -> state: in-progress` when the current workflow state is compatible.
- Add tests for state/accord synchronization and divergence warnings.
