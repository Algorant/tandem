---
id: task-230
type: task
title: "Make orphaned Board documents impossible at the protocol level"
state: "in-progress"
priority: "high"
references: ["task-228", "decision-7"]
relatedFiles: ["protocol/plan/spec.md", "tandem/src/app/tasks.rs", "tandem/src/app/support.rs", "tandem/src/cli/mod.rs"]
tags: ["protocol", "cli", "completion", "hierarchy", "bug"]
createdAt: "2026-08-17T12:19:27Z"
updatedAt: "2026-08-17T20:34:32Z"
accord:
  status: "claimed"
  assignee: "worker-task-230-0def6a42"
  claimedAt: "2026-08-17T20:34:32Z"
  updatedAt: "2026-08-17T20:34:32Z"
assignee: "worker-task-230-0def6a42"
---

## Description

## Goal

Make it impossible to create an orphan: an active Board document whose parent lives in `.tandem/logs/`. Prevention only, enforced at the two mutation entry points that can produce one.

## Invariant

> No Board document may have a parent that resides in Logs.

Equivalently: a task cannot be completed, canceled, or archived until every descendant is completed, canceled, or otherwise resolved; and a new task cannot be filed under a parent that is already archived.

## Observed defect

Reproduced in the `/home/ivan/.dotfiles/pi/.pi/.tandem` workspace under Epic `task-189`. `task-190` and `task-204` were completed and archived on 2026-08-15, but 11 of their Subtasks stayed active on the Board with `state: todo`:

- `task-190-1` … `task-190-4`
- `task-204-1` … `task-204-7`

Nothing reported this. `tandem list` printed them as ordinary rows with `RELATION subtask` and an archived `PARENT`. `tandem show task-190-1` printed `Subtask of: task-190` and `Location: board` with no warning.

That workspace has since been cleaned up manually and needs no further action here. Notably, per-child verification showed the parent summaries were not reliable evidence: 10 children were genuinely delivered, but `task-204-3` was not (pi-ask-me never adopted the shared harness) and was canceled and re-scoped as `task-235`. This is why no automatic repair is in scope.

## Root cause

`app/support.rs` (~line 259) validates a parent by existence alone:

```rust
if hierarchy.document(parent).is_none() {
    errors.push(format!("unresolved parentId `{parent}`"));
}
```

`hierarchy` spans Board and Logs, so an archived parent always passes. Two entry points exploit this:

1. **`complete`** (`app/tasks.rs`) archives a parent while descendants stay active. It performs no descendant check at all, while `cancel` in the same file already calls `active_task_descendant_ids` and hard-rejects. This caused the observed incident.
2. **`add --parent <archived-id>`** files a brand-new Board child under an already-archived parent.

Useful precedent: `unresolved_blockers_in_hierarchy` (`support.rs:159`) already reasons about location, treating an archived blocker as satisfied and an active Board blocker as unresolved. Parent validation simply never applied the same reasoning.

## Settled decisions

1. `complete` hard-rejects while any active task descendant exists, mirroring `cancel`. It does not cascade-archive. Cascading would invent `completion.summary` values in Logs, which the protocol defines as the terminal source of truth for work history; it would silently bury genuinely unfinished descendants, as `task-204-3` demonstrates; and it would conflate technical capability with actor authority (`protocol/plan/spec.md` line 982).
2. The rule covers every active task descendant, including direct Epic children, not only Subtasks beneath a Task.
3. `add` rejects a `parentId` that resolves to a Logs document. The parent must be on the Board.
4. Reparenting behavior is unchanged. Reparenting must never be used to clean up an orphan; that is guidance, not code.
5. Existing orphans are handled manually. This is a rare bug in an early project and does not warrant repair tooling.

## Implementation constraint

Enforce the archived-parent rule at the mutation entry points, **not** inside the shared document validator.

`complete` calls `validate_task_document_against_hierarchy` on the document being completed. If the archived-parent condition became a general error there, an existing orphan would fail validation during the exact operation needed to dispose of it, making it permanently stuck. The rule must block creating or moving a document into the orphan position, never block terminating a document already in it.

## Scope

Change normative semantics in `protocol/` first, then the Rust implementation, per AGENTS.md.

Protocol:

- State the invariant in `protocol/plan/spec.md`, and add the descendant precondition to the "Completion and logs" section, which currently lists five completion steps and no descendant rule.
- Reconcile line 655, which presently allows completing an Epic when "the human/project owner decides the Epic is done". Under decision 2 the owner must instead cancel each remaining child Task explicitly with a reason, which records the judgment per child rather than discarding it silently.
- Add error codes for the two rejections alongside the existing `E0xx` series.

Implementation:

- Add the `active_task_descendant_ids` precondition to `app::tasks::complete`, with an error message parallel to the existing `cancel` message.
- Require a Board-located parent when `add` resolves `--parent`.

## Acceptance criteria

- `tandem complete` refuses a task with any active task descendant and names them, matching the shape of the existing `cancel` error.
- The refusal applies both to an Epic with active child Tasks and to a Task with active Subtasks.
- Completing a task with no active descendants is unchanged.
- `tandem add --parent <archived-id>` is refused with a clear message.
- A pre-existing orphan can still be completed or canceled, so older workspaces remain repairable by hand.
- Tests cover: Task with active Subtasks, Epic with active child Tasks, clean completion, `add` under an archived parent, and disposal of a pre-existing orphan.
- `cargo fmt`, `cargo test`, and strict Clippy pass.

## Non-goals

- No orphan detection diagnostic, validation error, or `--orphaned` listing filter.
- No cascade-archive behavior and no opt-in cascade flag.
- No auto-repair command and no migration of existing workspaces.
- No change to `cancel`, which already enforces the descendant rule.
- No change to reparenting.

