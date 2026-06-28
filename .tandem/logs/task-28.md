---
id: task-28
type: task
title: "Define Validation as canonical workflow state"
priority: "high"
relatedFiles: ["protocol/plan/spec.md", "protocol/README.md", "README.md"]
tags: ["protocol", "validation", "state"]
createdAt: "2026-06-28T13:57:06Z"
updatedAt: "2026-06-28T15:31:34Z"
subtasks:
  - id: task-28-1
    title: "Replace default workflow state review with validation in protocol planning docs"
    completed: true
  - id: task-28-2
    title: "Define Validation as delivered work awaiting acceptance, rejection, redirection, or human/product judgment"
    completed: true
  - id: task-28-3
    title: "Clarify that review metadata remains distinct from workflow state"
    completed: true
  - id: task-28-4
    title: "Document temporary legacy handling for existing state: review files"
    completed: true
accord:
  status: "accepted"
  assignee: "review-validation-flow"
  claimedAt: "2026-06-28T15:05:06Z"
  deliveredAt: "2026-06-28T15:16:45Z"
  deliverables: ["docs:protocol/plan/spec.md:canonical default state changed from review to validation with clear semantics", "docs:protocol/README.md:protocol summary reflects validation workflow state", "docs:README.md:root summary updated if it states default states"]
  validation:
    commands: ["rg \"default states|state: review|review\" README.md protocol/README.md protocol/plan/spec.md"]
  constraints: ["Planning/spec update only; do not implement CLI/TUI behavior in this task.", "Do not rename review metadata unless a separate explicit decision is made.", "Keep blocked/failed/rework as attention signals, not automatic validation-state membership."]
  summary: "Protocol docs now define validation as canonical workflow state, with review metadata preserved and legacy state: review tolerance documented."
  evidence: ["rg validation/review command passed with remaining review references intentionally metadata/legacy/doc-history."]
  filesChanged: ["protocol/plan/spec.md", "protocol/README.md", "README.md"]
  reviewer: "ivan"
  note: "Accepted as completed foundation for the Validation direction; remaining board/TUI cleanup is tracked separately."
  updatedAt: "2026-06-28T15:31:34Z"
completedAt: "2026-06-28T15:31:34Z"
completion:
  summary: "Defined Validation as the canonical workflow state in protocol/root docs while preserving review metadata and legacy review-state tolerance."
  validation: "review-validation-flow validation passed; accepted by orchestrator as first Validation direction"
  reviewer: "ivan"
---

## Description

Replace the default `review` workflow state with `validation` in the protocol plan before implementation hardens.

Validation means delivered work awaiting acceptance, rejection, requested changes, automated evidence review, or human/product judgment. Existing `review:` metadata remains the place to record reviewer decision/status unless a later task renames it.

Compatibility expectation: existing `state: review` files should be tolerated as legacy during transition; new writes should prefer `state: validation` after implementation tasks land.
