---
id: task-229
type: task
title: "Prevent tandem complete from orphaning active descendants"
priority: "high"
references: ["task-228", "decision-7", "decision-8"]
relatedFiles: ["protocol/plan/spec.md", "tandem/src/app/tasks.rs", "tandem/src/app/support.rs", "tandem/src/protocol/diagnostic.rs", "tandem/src/cli/mod.rs"]
tags: ["protocol", "cli", "completion", "hierarchy", "validation", "bug"]
createdAt: "2026-08-17T04:12:34Z"
updatedAt: "2026-08-17T11:56:13Z"
completedAt: "2026-08-17T11:56:13Z"
completion:
  outcome: "canceled"
  summary: "Canceled: Created in error. The tandem complete descendant-orphaning fix is being handled elsewhere, and this repository task was not requested."
---

## Description

## Goal

Make it impossible for `tandem complete` to archive a task while its descendants stay active on the Board, and make already-orphaned documents visible to structural validation.

## Observed defect

Reproduced in the `/home/ivan/.dotfiles/pi/.pi/.tandem` workspace under Epic `task-189`:

- `task-190` is archived in `.tandem/logs/task-190.md`, but `task-190-1` … `task-190-4` remain on the Board with `state: todo`.
- `task-204` is archived in `.tandem/logs/task-204.md`, but `task-204-1` … `task-204-7` remain on the Board with `state: todo`.

Both parent completion summaries show the Subtask work was actually done and integrated. The Board therefore advertises 11 `todo` items whose work is already complete and archived.

Nothing reports this. `tandem list` prints the orphans as ordinary rows with `RELATION subtask` and `PARENT task-190`. `tandem show task-190-1` prints `Subtask of: task-190` and `Location: board` with no warning.

## Root cause

Two independent gaps:

1. `tandem/src/app/tasks.rs::complete` performs no descendant check. `cancel` in the same file calls `active_task_descendant_ids` and hard-rejects with `cannot cancel <id> while it has active descendants: …`, but `complete` only validates the document, checks unresolved blockers, emits completion-policy warnings, then archives.
2. `parentId` resolution scans Board *and* Logs, so an archived parent still resolves. An orphaned Board document is structurally invisible to lint.

## Decisions taken

- Completion hard-rejects, mirroring `cancel`. It does not cascade-archive. Cascading would require inventing `completion.summary` values in Logs, which the protocol defines as the terminal source of truth for work history; it would silently discard genuinely unfinished descendants; and it would conflate technical capability with actor authority (`protocol/plan/spec.md` line 982).
- The rule covers **every** active task descendant, including direct Epic children, not only Subtasks beneath a Task. One uniform invariant: *an archived task never has active descendants.*

## Scope

Change normative semantics in `protocol/` first, then the Rust implementation, per AGENTS.md.

Protocol:

- Add the descendant precondition to the "Completion and logs" section of `protocol/plan/spec.md`, which currently lists five completion steps and no descendant rule.
- Reconcile line 655. It presently allows completing an Epic when "the human/project owner decides the Epic is done", which the chosen uniform scope removes. The owner must now cancel each leftover child Task explicitly with a reason, which records the decision per child instead of discarding it silently.
- Add an error code for the orphaned-document condition alongside the existing `E0xx` series.

Implementation:

- Add the `active_task_descendant_ids` precondition to `app::tasks::complete`, with an error message parallel to the existing `cancel` message.
- Add a structural diagnostic that treats a Board document whose resolved parent lives in Logs as an error, so `list`, `show`, and validation surface pre-existing corruption.

## Acceptance criteria

- `tandem complete` refuses a task that has any active task descendant, and names them, matching the shape of the `cancel` error.
- The refusal applies to an Epic with active child Tasks and to a Task with active Subtasks.
- Completing a task with no active descendants is unchanged.
- A Board document whose parent resolves to Logs is reported as a structural error by validation and by read commands.
- Tests cover: Task with active Subtasks, Epic with active child Tasks, clean completion, and detection of a pre-existing orphan.
- `cargo fmt`, `cargo test`, and strict Clippy pass.

## Repair

The `.dotfiles/pi` workspace is a separate repository and is not fixed by this task. Once the diagnostic ships, dispose of the 11 orphans there explicitly: complete the ones whose work the parent summaries show as done, and cancel any that are genuinely obsolete.

## Open question

The uniform scope changes a normative allowance at spec line 655. Confirm whether this warrants its own `type: decision` record, or whether the protocol edit plus this task is sufficient.

## Non-goals

- No cascade-archive behavior and no new opt-in flag.
- No automatic repair or migration of existing orphaned workspaces.
- No change to `cancel`, which already enforces the invariant.

