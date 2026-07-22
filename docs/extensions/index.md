---
title: Extensions
description: Tandem integration adapters.
---
Tandem integrations should be thin adapters over the installed `tandem` CLI.

## Adapter principle

```text
LLM / editor agent → integration adapter → tandem CLI → .tandem workspace
```

Adapters own tool schemas, editor or agent ergonomics, output formatting, and diagnostics. They should not duplicate Tandem protocol parsing or mutation behavior.

## Current adapter

`extensions/pi-tandem/` provides a Pi adapter that exposes Tandem task, accord, rule, decision, and log operations through Pi tools and commands. Agents should use `tandem_decision` for ADR-compatible decisions instead of inventing task states or a separate ADR type.

### Create a Task campaign

With Pi, pass `kind` and `parent` through to Tandem and consume the IDs and relationships returned by the CLI:

```ts
// Returns a global Epic ID, for example task-103.
tandem_task({ action: "add", title: "Coordinate the release", kind: "epic" })

// A direct Epic child is a global Task, for example task-104.
tandem_task({ action: "add", title: "Write release notes", parent: "task-103" })

// A direct Task child is a parent-derived leaf Subtask, for example task-104-1.
tandem_task({ action: "add", title: "Check upgrade notes", parent: "task-104" })
```

The adapter never allocates IDs or reclassifies relationships. Tandem returns `epic-task` for the global Task beneath the Epic, `subtask` for the parent-derived Subtask, and generic `parent` for a global Task attached to a decision or custom document.

Only global-ID Tasks are initial Shep delegation roots. Delegate the Task—not its Epic or Subtask:

```ts
shep_delegate({
  taskId: "task-104",
  checkoutMode: "worktree",
})
```

One worker owns the delegated Task's direct Subtasks as its session todo projection and produces one Task-root handoff. Epics and Subtasks are not independently delegated. The adapter does not expose deprecated inline checklist `subtasks` authoring, and there is no compatibility path for hierarchical direct Epic children, global-ID Subtasks, or deeper nesting.

Global Pi config promotion is a separate explicit task. Repository-local extension development should not edit `~/.pi/agent` directly.
