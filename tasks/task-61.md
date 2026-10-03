---
id: task-61
uid: 2b525ff9-c1f8-4995-8b86-e3e23d5bac5c
type: task
title: "update --clear parent on a subtask bypasses the role-change guard and writes an invalid board"
state: todo
priority: "medium"
effort: "small"
relatedFiles: ["tandem/src/app/tasks.rs", "tandem/tests/cli_behavior.rs"]
tags: ["protocol", "validation"]
accord:
  status: "ready"
  acceptance: ["`tandem update <subtask> --clear parent` fails before writing, with the same canonical-role/IDs-are-immutable error as `--parent`, and the record file is unchanged.", "Clearing the parent on a root Task, and valid same-role reparenting, behave as before.", "A regression test covers clear-parent on a subtask (refused, board still readable) and on a root Task."]
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-03T03:11:26Z"
createdAt: "2026-10-03T03:11:26Z"
updatedAt: "2026-10-03T03:11:26Z"
---

## Description


## Description

Reported from the ~/.pi board (task-337), observed live on 2026-10-02 and reproduced on 0.16.2 in disposable workspaces on 2026-10-03.

`tandem update <task-N-M> --clear parent` returns `ok` and writes the subtask with no parent. The record then fails validation ("task … has invalid ID `task-N-M`; expected global `task-N`"). After that, `show`, unrelated updates, and `update <id> --parent task-N` all fail with the same validation error. This happens on Git-backed boards (sync reports it as held) and on not-git boards. On ~/.pi the only repair was a user-run restore of the file from origin/tandem.

The other role changes are already refused before writing ("reparenting … would change its canonical role …; IDs are immutable"): `--parent task-2` on a subtask of task-1, and `--parent task-1` on a root Task.

## Cause

In `tandem/src/app/tasks.rs` `update()`, the prospective document applies only `options.parent`, and the role-change guard is gated on `options.parent.is_some()`. A `--clear parent` therefore skips both the prospective role computation and the guard. The clear is applied later, through the `"parent" => "parentId"` clear mapping.

