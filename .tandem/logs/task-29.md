---
id: task-29
type: task
title: "Update CLI for validation state compatibility"
priority: "high"
relatedFiles: ["tandem/src/main.rs", "tandem/plan/spec.md"]
tags: ["cli", "validation", "state"]
blockers: ["task-28"]
createdAt: "2026-06-28T13:57:07Z"
updatedAt: "2026-06-28T15:31:34Z"
subtasks:
  - id: task-29-1
    title: "Change init/default workspace states to todo, in-progress, validation"
    completed: true
  - id: task-29-2
    title: "Accept existing state: review as a legacy alias without breaking reads"
    completed: true
  - id: task-29-3
    title: "Prefer validation for new writes, moves, examples, and tests"
    completed: true
  - id: task-29-4
    title: "Update list/show/search/move behavior and JSON expectations as needed"
    completed: true
accord:
  status: "accepted"
  assignee: "review-validation-flow"
  claimedAt: "2026-06-28T15:08:28Z"
  deliveredAt: "2026-06-28T15:16:45Z"
  deliverables: ["code:tandem/src/main.rs:CLI defaults and state handling prefer validation while tolerating legacy review", "docs:tandem/plan/spec.md:CLI command reference updated for validation", "tests:tandem:coverage for validation defaults and review legacy alias"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test"]
  constraints: ["Do not perform broad repository migration unless explicitly included by the orchestrator.", "Keep workflow state distinct from review metadata and accord validation commands."]
  summary: "CLI defaults and state handling now prefer validation while accepting legacy review aliases; common accord transitions synchronize compatible workflow states."
  evidence: ["cargo fmt --check and cargo test passed; smoke add/list/show/complete with validation state succeeded."]
  filesChanged: ["tandem/src/main.rs", "tandem/plan/spec.md"]
  reviewer: "ivan"
  note: "Accepted as completed foundation for the Validation direction; remaining board/TUI cleanup is tracked separately."
  updatedAt: "2026-06-28T15:31:34Z"
completedAt: "2026-06-28T15:31:34Z"
completion:
  summary: "Updated CLI defaults and state handling to prefer validation while tolerating legacy review aliases and syncing common accord transitions."
  validation: "review-validation-flow validation passed; accepted by orchestrator as first Validation direction"
  reviewer: "ivan"
---

## Description

Make `tandem` create and prefer `state: validation` while safely handling existing `state: review` as legacy/alias during the transition.

This should cover init defaults, state validation, list/show/search/move behavior, command examples, and tests.
