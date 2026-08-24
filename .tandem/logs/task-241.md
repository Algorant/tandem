---
id: task-241
type: task
title: "Surface delivered-but-untriaged work on the Board"
priority: "high"
effort: "small"
references: ["decision-11", "task-239", "task-239-2"]
relatedFiles: ["tandem/src/tui/board/mod.rs"]
tags: ["tui", "board", "workflow", "review", "visual"]
createdAt: "2026-08-24T23:04:21Z"
updatedAt: "2026-08-24T23:46:00Z"
accord:
  status: "accepted"
  assignee: "worker-task-241-f93d9c4b"
  claimedAt: "2026-08-24T23:16:14Z"
  deliveredAt: "2026-08-24T23:45:34Z"
  validation:
    commands: ["cargo fmt", "cargo clippy --all-targets -- -D warnings", "cargo test: 279 unit + 11 integration passed", "orchestrator terminal validation via Herdr pane against a purpose-built fixture workspace, verdigris theme"]
  summary: "Board now surfaces delivered-but-untriaged work. Removed the dead Document parameter from board_should_surface_accord_status and moved state-dependent chip precedence to the call site. Added a Delivered · untriaged filter reachable via f, with its own clear option and filter-bar chip. Added test coverage for the predicate near misses, combined filters, chip precedence, and badge_disabled suppression."
  filesChanged: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/pickers.rs"]
  note: "Orchestrator-verified including terminal rendering, which was the outstanding blocker. Ran the built binary in a Herdr pane against a four-case fixture and confirmed: in-progress delivered vs non-delivered rows are distinct (DELIVERED chip cream #ebdbb2 on dark green #40503e, adjacent to WIP on brown #513a2c); a validation row with delivered accord and pending review shows PENDING and not DELIVERED; the Delivered · untriaged filter reduces counts to TODO 0 / IN PROGRESS 1 / VALIDATION 0 and isolates the correct task; clear all filters restores the board. Top-level Board (4) count not reflecting filters is pre-existing view-total behavior, not introduced here."
  updatedAt: "2026-08-24T23:45:46Z"
assignee: "worker-task-241-f93d9c4b"
completedAt: "2026-08-24T23:46:00Z"
completion:
  summary: "Board surfaces delivered-but-untriaged work, resolving decision-11 E1 and unblocking task-239-2. Chip precedence in validation favors the pending review; in-progress delivered work carries a DELIVERED chip. Added a Delivered · untriaged filter via f with clear options and filter-bar indication, plus test coverage for near misses, combined filters, precedence, and badge_disabled. Verified in a real terminal against a four-case fixture. Merged to main."
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
