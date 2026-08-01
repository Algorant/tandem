---
id: decision-7
type: decision
title: "Distinguish Epic, Task, and Subtask roles and nomenclature"
status: "accepted"
date: "2026-07-15"
deciders: ["Algorant"]
context: "Tandem conflated every task-to-task child with a Subtask and decision-4 allocated parent-derived IDs beneath Epics. The canonical product model requires a strict Epic → Task → Subtask boundary in both relationships and names: Epics contain globally numbered Tasks, while only Tasks contain parent-derived Subtasks."
consequences: ["Epics and Tasks use global task-N IDs; only Subtasks use parent-derived task-N-M IDs.", "Direct Epic children are Tasks with global IDs and canonical parentId links to the Epic.", "Only Task-role documents are delegatable; their Subtasks become the delegated worker's execution checklist.", "Epics cannot have parents, Subtasks cannot have children, and role-changing reparenting that violates nomenclature is rejected.", "Decision-4 is fully superseded with no compatibility exception for its incorrect Epic-child allocation.", "Protocol, CLI, TUI, Pi integration guidance/tests, active planning trees, and public documentation must be corrected consistently."]
alternatives: ["Keep treating direct Epic children as Subtasks; rejected because Epics contain Tasks.", "Keep hierarchical IDs for direct Epic Tasks while changing only labels; rejected because task-N-M nomenclature is reserved for Subtasks.", "Add separate protocol document types; rejected because roles remain derivable over normal task documents.", "Retain decision-4 compatibility behavior; rejected because the allocation error must not remain canonical or supported.", "Allow Epic or Subtask delegation; rejected because only Tasks own executable Subtask checklists in the initial model."]
supersedes: ["decision-4"]
references: ["decision-4", "task-101", "task-132", "task-133", "task-134"]
tags: ["protocol", "hierarchy", "epics", "tasks", "subtasks", "ids", "delegation"]
createdAt: "2026-07-15T17:42:24Z"
updatedAt: "2026-07-15T19:38:54Z"
---

## Status

Accepted. This decision fully supersedes decision-4.

## Canonical model

Tandem has three hierarchy roles while retaining one task document type:

1. **Epic** — `type: task` plus `kind: epic`, using a global `task-N` ID.
2. **Task** — a normal `type: task` document using a global `task-N` ID. It may be root-level, have a generic non-task parent, or be a direct child of an Epic.
3. **Subtask** — a normal `type: task` document directly beneath a Task, using the parent-derived `task-N-M` designation.

Example:

```text
task-134       Epic
├── task-139   Task (`parentId: task-134`)
│   ├── task-139-1   Subtask (`parentId: task-139`)
│   └── task-139-2   Subtask (`parentId: task-139`)
└── task-140   Task (`parentId: task-134`)
```

Canonical `parentId` defines the relationship. The ID boundary defines the required nomenclature and default allocation; it is not a substitute for resolving `parentId`.

## Allocation

- Creating an Epic or root Task allocates the next global `task-N`.
- Creating a Task under an Epic also allocates the next global `task-N`, then writes the Epic's ID as `parentId`.
- Creating a Task with a decision/custom-document parent allocates the next global `task-N` and writes the generic `parentId`.
- Creating a Subtask under a Task allocates the next unused `task-N-M` across active and completed work.
- Creating a child beneath a Subtask is invalid.
- Creating or reparenting an Epic beneath any parent is invalid.

Task and Subtask IDs remain immutable after valid creation. Reparenting is allowed only when the existing ID remains valid for the resulting role and nomenclature. A Task may move between root and Epic parents because it retains a global ID. A Task cannot become a Subtask without replacement, and a Subtask cannot become a Task or move to a different Task parent without replacement.

## Classification and output

Relationship output distinguishes:

- `epic-task` for a global Task whose resolved parent is an Epic;
- `subtask` for a parent-derived Subtask whose resolved parent is a Task;
- generic `parent` for decision/custom-document relationships.

Showing an Epic exposes **Tasks**. Showing a Task exposes **Subtasks**. CLI, TUI, Review, Logs, search, Pi integrations, and documentation use the same resolved classifier and must not call a direct Epic child a Subtask.

## Delegation

Only Task-role documents are delegation roots in the initial model.

- An Epic is not delegated; its Tasks are delegated independently.
- Delegating a Task gives Worker A ownership of that Task and its direct Subtasks as a structured execution checklist.
- A Subtask is not independently delegated.
- Nested Worker B delegation is deferred.

Tandem remains the durable task-tree source of truth. Pi todo state may project the delegated Task's Subtasks for live worker/widget progress without changing their first-class Tandem representation.

## Structural constraints

- Epics are root-only.
- Epics contain Tasks, never direct Subtasks.
- Tasks may contain Subtasks.
- Subtasks are leaf-only.
- Missing parents, cycles, nested Epics, role/ID mismatches, and children beneath Subtasks are structural errors.

## No compatibility exception

Decision-4's direct-Epic hierarchical allocation was an error and is not retained as a legacy behavior. Incorrect active planning records are replaced with canonical global Task IDs before implementation proceeds. Tandem must not add fallback classification, migration shims, or dynamic acceptance for that invalid form.

## Consequences

- Active task-133 and task-134 child records created under the superseded rule must be rebuilt as globally numbered Tasks while preserving their intended content and dependencies.
- Protocol, project instructions, allocation, validation, CLI/TUI output, repository pi-tandem guidance, tests, and public documentation must change together.
- Historical discussion may describe the superseded rule, but no current implementation path depends on it.
