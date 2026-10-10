---
id: task-61
uid: 2b525ff9-c1f8-4995-8b86-e3e23d5bac5c
type: task
title: "update --clear parent on a subtask bypasses the role-change guard and writes an invalid board"
priority: "medium"
effort: "small"
relatedFiles: ["tandem/src/app/tasks.rs", "tandem/tests/cli_behavior.rs"]
tags: ["protocol", "validation"]
accord:
  status: "accepted"
  acceptance: ["`tandem update <subtask> --clear parent` fails before writing, with the same canonical-role/IDs-are-immutable error as `--parent`, and the record file is unchanged.", "Clearing the parent on a root Task, and valid same-role reparenting, behave as before.", "A regression test covers clear-parent on a subtask (refused, board still readable) and on a root Task."]
  claimedAt: "2026-10-10T14:09:46Z"
  deliveredAt: "2026-10-10T14:12:18Z"
  validation: ["$ just dev-check"]
  summary: "`update --clear parent` now goes through the same prospective-document role computation and canonical-role guard as `--parent`, in the shared app path (tandem/src/app/tasks.rs update()). Clearing the parent on a subtask is refused before any write; root Task and Epic-child Task clears behave as before. Two regression tests added; just dev-check passes."
  evidence: ["`update <subtask> --clear parent` fails before writing with the canonical-role/IDs-are-immutable error and the record is unchanged: Test update_clear_parent_on_a_subtask_is_refused_without_writing: stderr contains 'would change its canonical role from subtask to task' and 'IDs are immutable'; record file bytes identical before/after; show, update --priority and list still succeed afterwards.", "Clearing parent on a root Task and valid same-role reparenting behave as before: Test update_clear_parent_on_root_task_and_epic_task_still_works: clear on root Task succeeds as a no-op with file unchanged; clear on an Epic-child Task succeeds, removes parentId, and show still works. Other existing --parent suites (kind_behavior, epic_completion, assignment) pass in dev-check.", "Regression test covers clear-parent on a subtask and on a root Task: Both tests added in tandem/tests/cli_behavior.rs and pass; subtask test fails without the fix."]
  filesChanged: ["tandem/src/app/tasks.rs", "tandem/tests/cli_behavior.rs"]
  updatedAt: "2026-10-10T14:12:19Z"
createdAt: "2026-10-03T03:11:26Z"
updatedAt: "2026-10-10T14:12:19Z"
assignee: "worker-task-61-ed95abf3"
archivedAt: "2026-10-10T14:12:19Z"
resolution:
  outcome: "completed"
---

## Description


## Description

Reported from the ~/.pi board (task-337), observed live on 2026-10-02 and reproduced on 0.16.2 in disposable workspaces on 2026-10-03.

`tandem update <task-N-M> --clear parent` returns `ok` and writes the subtask with no parent. The record then fails validation ("task … has invalid ID `task-N-M`; expected global `task-N`"). After that, `show`, unrelated updates, and `update <id> --parent task-N` all fail with the same validation error. This happens on Git-backed boards (sync reports it as held) and on not-git boards. On ~/.pi the only repair was a user-run restore of the file from origin/tandem.

The other role changes are already refused before writing ("reparenting … would change its canonical role …; IDs are immutable"): `--parent task-2` on a subtask of task-1, and `--parent task-1` on a root Task.

## Cause

In `tandem/src/app/tasks.rs` `update()`, the prospective document applies only `options.parent`, and the role-change guard is gated on `options.parent.is_some()`. A `--clear parent` therefore skips both the prospective role computation and the guard. The clear is applied later, through the `"parent" => "parentId"` clear mapping.

