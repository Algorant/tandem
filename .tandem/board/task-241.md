---
id: task-241
type: task
title: "Surface delivered-but-untriaged work on the Board"
state: "in-progress"
priority: "high"
effort: "small"
references: ["decision-11", "task-239", "task-239-2"]
relatedFiles: ["tandem/src/tui/board/mod.rs"]
tags: ["tui", "board", "workflow", "review", "visual"]
createdAt: "2026-08-24T23:04:21Z"
updatedAt: "2026-08-24T23:16:14Z"
accord:
  status: "claimed"
  assignee: "worker-task-241-f93d9c4b"
  claimedAt: "2026-08-24T23:16:14Z"
  updatedAt: "2026-08-24T23:16:14Z"
assignee: "worker-task-241-f93d9c4b"
---

## Description

## Why

Resolves E1 from `decision-11`, which blocks task-239-2.

Under decision-11, no accord action moves workflow state except `claim`. Delivered work therefore sits in `in-progress` with `accord: delivered`, awaiting orchestrator triage, instead of sitting visibly in `validation`. If the Board does not distinguish it from work still being written, a visible parking lot is replaced by an invisible one and the change is a net regression.

## Existing machinery

Most of this is already built. Confirm before writing anything new.

- `board_should_surface_accord_status` (`tandem/src/tui/board/mod.rs:2130`) already renders a `delivered` chip.
- `board_should_surface_review_status` (`mod.rs:2168`) already renders a `pending` chip.

Both are theme-disableable via `badge_disabled`.

## Work

1. **Remove the inverted suppression rule.** `board_should_surface_accord_status` currently returns `false` when `document_state_label(doc) == "validation" && normalized == "delivered"`, because that pairing was redundant under the old semantics. Under decision-11 a task in `validation` has `accord: delivered` plus `review: pending`, so decide deliberately which chip wins there rather than leaving a rule written for the previous model.

2. **Verify the `in-progress` + `delivered` case reads clearly.** This is the case E1 is about. A delivered task must be immediately distinguishable from an actively-worked one at a glance, in a real terminal, in both light and dark themes.

3. **Add a triage affordance.** A filter or view for work that is delivered and untriaged, so it can be found without scanning the whole board. Keep it a simple filtered list, consistent with the v0 decision that the review queue is not hard-coded workflow sections.

## Acceptance criteria

- A task in `in-progress` with `accord: delivered` is visually distinct from one without, at a glance.
- A task in `validation` with `review: pending` is visually distinct, and the chosen chip precedence there is deliberate and documented in a comment.
- Delivered-and-untriaged work is reachable through a filter or view without scanning.
- Existing theme `badge_disabled` opt-outs continue to work.
- Human terminal validation in both light and dark themes. Automated tests do not substitute for this.

## Constraints

- Follow repository AGENTS.md.
- This is a visible TUI change and requires human terminal validation before acceptance. Do not accept it on automated evidence alone.
- Do not add new workflow states or hard-coded workflow sections.
