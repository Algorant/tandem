---
id: task-31
type: task
title: "Refresh docs for Validation workflow"
priority: "med"
relatedFiles: ["README.md", "protocol/README.md", "tandem/README.md", "extensions/README.md", "tandem/plan/todo.md"]
tags: ["docs", "validation"]
blockers: ["task-28", "task-29", "task-30"]
createdAt: "2026-06-28T13:57:07Z"
updatedAt: "2026-06-28T17:33:50Z"
subtasks:
  - id: task-31-1
    title: "Update root and protocol READMEs for Validation state wording"
    completed: true
  - id: task-31-2
    title: "Update CLI/TUI README and planning todo references"
    completed: false
  - id: task-31-3
    title: "Remove stale top-level Review pane language from docs"
    completed: false
  - id: task-31-4
    title: "Check remaining review references mean review metadata or legacy context"
    completed: true
accord:
  status: "claimed"
  assignee: "review-validation-flow"
  claimedAt: "2026-06-28T15:16:45Z"
  deliverables: ["docs:README.md:root workflow summary uses Validation", "docs:protocol/README.md:protocol summary uses Validation", "docs:tandem/README.md:current TUI behavior and key help updated", "docs:tandem/plan/todo.md:task checklist reflects Validation direction"]
  validation:
    commands: ["rg \"Review|review|validation|Validation\" README.md protocol/README.md tandem/README.md extensions/README.md tandem/plan/todo.md"]
  constraints: ["Keep docs concise; do not rewrite unrelated planning sections.", "Do not remove review metadata terminology where it is still intentionally distinct."]
  updatedAt: "2026-06-28T15:16:45Z"
completedAt: "2026-06-28T17:33:50Z"
completion:
  summary: "Refreshed CLI/TUI documentation for Validation as a Board workflow state and removed stale top-level Review pane language from current docs."
  validation: "rg Review README.md protocol/README.md tandem/README.md extensions/README.md tandem/plan/todo.md"
---

## Description

Update user-facing and planning docs so Validation is the board workflow state and Review references only reviewer/review metadata or legacy context.

Keep wording concise and avoid reintroducing a top-level Review pane.
