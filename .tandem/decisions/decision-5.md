---
id: decision-5
type: decision
title: "Distinguish Epic, Task, and Subtask roles and nomenclature"
status: "accepted"
deciders: ["Algorant"]
tags: ["protocol", "hierarchy", "epics", "tasks", "subtasks", "ids", "delegation"]
createdAt: "2026-08-31T20:52:47Z"
decidedAt: "2026-07-14T00:00:00Z"
updatedAt: "2026-08-31T20:52:47Z"
---

## Status

Accepted. This decision defines the Epic, Task, and Subtask hierarchy retained by protocol 0.3.0. It supersedes the earlier subtask-ID allocation approach, which is not retained in this workspace (see the archived pre-0.3.0 workspace under `.tandem_old`).

## Context

Tandem needs exactly three hierarchy roles over one task document type, with role derived from resolved documents and ID shape constrained by the derived role.

## Decision

1. **Epic** — `type: task` plus `kind: epic`, global `task-N` ID, root-only (no parentId).
2. **Task** — normal `type: task`, global `task-N` ID; root-level or a direct Epic child (relationship `epic-task`).
3. **Subtask** — direct child of a Task only (relationship `subtask`), parent-derived `task-N-M` ID, leaf (no children).

Only Task-role documents are delegatable. Subtasks cannot be reparented; moving the work means canceling and recreating it. Role-changing reparenting and role/ID mismatches are structural errors.

## Consequences

- Epics and Tasks share the global `task-N` allocator; only Subtasks use parent-derived IDs.
- Hierarchy renders as a bounded three-level tree without recursive role logic.
- Reparenting is valid only when it does not change a document's role or invalidate its canonical ID.
