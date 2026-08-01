---
id: decision-4
type: decision
title: "Allocate parent-derived hierarchical IDs for new first-class subtasks"
status: "superseded"
date: "2026-07-14"
deciders: ["Algorant"]
context: "Decision-4 introduced parent-derived IDs but incorrectly applied them to every task-parent relationship, including direct children of Epics. Decision-7 establishes the canonical Epic → Task → Subtask boundary and reserves hierarchical task-N-M IDs for Subtasks only."
consequences: ["Decision-4 must not be used as current allocation guidance.", "Decision-7 exclusively defines current Epic, Task, Subtask, naming, delegation, and reparenting rules.", "Incorrect active Epic children created with hierarchical IDs are replaced rather than retained through compatibility behavior."]
alternatives: ["Retain decision-4 as a legacy exception; rejected because it would preserve the exact Epic/Task/Subtask ambiguity decision-7 resolves."]
supersededBy: ["decision-7"]
references: ["task-101", "task-104", "task-106", "task-103", "decision-7"]
tags: ["protocol", "subtasks", "relationships", "ids"]
createdAt: "2026-07-14T00:53:44Z"
updatedAt: "2026-07-15T19:38:54Z"
---

## Status

Superseded by decision-7.

## Historical context

Decision-4 correctly established that first-class Subtasks are full Tandem task documents and that their IDs should use parent-derived designations. It incorrectly generalized that allocation rule to every task parent, including Epics. That produced direct Epic children such as `task-134-1`, even though this nomenclature is reserved for a Subtask of `task-134`.

## Supersession

Decision-7 replaces decision-4 in all current behavior. There is no compatibility or legacy exception for direct Epic children with Subtask-shaped IDs.

Current allocation is:

- Epic or Task: globally allocated `task-N`.
- Task directly under an Epic: globally allocated `task-N` plus canonical `parentId` pointing to the Epic.
- Subtask directly under a Task: parent-derived `task-N-M` plus canonical `parentId` pointing to the Task.

Incorrect active records created under decision-4 are replaced with canonical Tasks rather than dynamically reclassified or retained as legacy IDs.
