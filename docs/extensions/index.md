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

### Create and delegate child tasks

With Pi, create a normal child task by passing the parent task's ID:

```ts
tandem_task({
  action: "add",
  title: "Write release notes",
  parent: "task-103",
})
```

The adapter forwards `parent` to the Tandem CLI as `parentId`. The CLI validates the relationship and returns the allocated ID, such as `task-103-1`; the adapter does not generate IDs. Read that returned ID, then delegate the existing child with Shep:

```ts
shep_delegate({
  taskId: "task-103-1",
  checkoutMode: "worktree",
})
```

Shep receives the existing `taskId`; it does not allocate or forward a parent field. Use a child task when delegated work needs its own owner, accord, review, or completion lifecycle; do not create new inline checklist `subtasks` for delegation.

Global Pi config promotion is a separate explicit task. Repository-local extension development should not edit `~/.pi/agent` directly.
