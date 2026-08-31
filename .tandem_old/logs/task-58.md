---
id: task-58
type: task
title: "Add Validation apply-accepted workflow"
priority: "medium"
references: ["task-55"]
relatedFiles: ["tandem/src/tui.rs", "tandem/src/main.rs", "tandem/src/tui/review.rs"]
tags: ["tui", "validation", "review", "logs", "ux"]
createdAt: "2026-06-29T17:22:29Z"
updatedAt: "2026-06-29T20:11:14Z"
accord:
  status: "accepted"
  deliveredAt: "2026-06-29T18:09:29Z"
  deliverables: ["Added Apply accepted prompt and footer/help surface for accepted Validation items.", "Added candidate filtering and completion/log movement for accepted-only Validation tasks.", "Added tests for candidate selection, cancel/no-op, and confirmed log archival excluding delivered items."]
  validation:
    commands: ["cd tandem && cargo fmt --check && cargo test"]
  summary: "Implemented explicit Validation Apply accepted workflow: C/footers now open an Apply accepted confirmation dialog listing accepted task ids/titles, accepted candidates are selected only when state=validation plus accord.status/review.status accepted, cancel is no-op, and confirm completes only those candidates into logs using existing completion metadata/event behavior."
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "tui"
  updatedAt: "2026-06-29T20:11:10Z"
review.decidedAt: "2026-06-29T20:11:10Z"
review.reviewer: "tui"
review.status: "accepted"
completedAt: "2026-06-29T20:11:14Z"
completion:
  summary: "Applied accepted Validation sign-off for task-58"
  validation: "Accepted by Validation apply-accepted workflow"
  reviewer: "tui"
---

## Description

Add an explicit workflow for applying accepted Validation items to completed logs. The current per-item `C` affordance is unclear; accepted validation candidates should instead be reviewed in a clear apply/archive dialog before they leave the Board.

Scope:
- Validation pane should distinguish delivered items awaiting sign-off from accepted items ready to be applied/logged.
- Add an explicit `Apply accepted` / `Archive accepted` action surface for accepted validation candidates.
- Opening the action should show a confirmation dialog listing the accepted task IDs/titles that will be completed and moved to logs.
- Confirming should complete/archive only accepted validation candidates; delivered-but-unaccepted and rework items must not be included.
- Canceling should be a no-op.
- This should make the old per-item `C` behavior unnecessary or clearly de-emphasized.

Design notes:
- Coordinate with task-55: accept/rework modal behavior, badge semantics, and modal input isolation all interact with this workflow.
- Keep workflow state, review metadata, accord status, and completion/log movement distinct.
- The dialog should explain exactly what will happen before mutating tasks.
- Prefer batch apply for accepted work over accidental single-key completion.

Acceptance:
- Accepted Validation items are visibly distinguishable as ready to apply/log.
- Apply dialog lists affected task IDs/titles and excludes unaccepted delivered items.
- Confirm moves accepted candidates to logs using existing completion behavior.
- Cancel leaves all task files unchanged.
- Tests cover candidate selection, cancel/no-op, confirmed completion/log movement, and interaction with task-55 accept/rework states.
