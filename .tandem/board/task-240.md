---
id: task-240
type: task
title: "Collapse adjacent metadata-only commits before push"
state: todo
priority: "low"
effort: "small"
references: ["task-228", "task-239"]
relatedFiles: ["justfile", "AGENTS.md"]
tags: ["git", "tooling", "workflow", "automation"]
createdAt: "2026-08-24T22:32:59Z"
updatedAt: "2026-08-24T22:32:59Z"
---

## Description

## Goal

Add a pre-push step that combines adjacent auto-generated `.tandem/` metadata commits into one, so local history stays readable without disabling the auto-committer.

## Background

`.tandem/` state is committed automatically as `chore(tandem): checkpoint metadata` while work proceeds. This is worth keeping: it means board, log, and event state is never lost mid-session. The cost is one commit per checkpointer run.

The 2026-08-22 push carried 11 commits, 6 of them checkpoint metadata:

```
a074677 meta    8a00d1a src     a1cbb7c meta    98914a4 src
2850da9 meta    a1f7f81 src     56636c1 meta    8907705 meta
96aff55 meta    f8f4b23 meta
```

## Rule

Before pushing, squash any run of **consecutive** commits touching only `.tandem/` into one `coord(tandem):` commit.

- Never touch a commit that modifies source.
- Never reorder across a source commit, so isolated metadata commits stay where they are.
- Never rewrite already-pushed history.

On the range above this collapses the trailing run of four into one, giving 8 commits instead of 11. The three isolated metadata commits stay put.

## Why not group by task

An earlier proposal was to squash each task's intermediate commits together. That does not work here. Worker branches are already squashed at integration (`a1f7f81` is literally "Squash commits from worker-task-237-..."), and the checkpointer's commits are grouped by when it ran, not by task: `8907705` archives task-237 while modifying task-238, and `2850da9` creates four task-239 subtasks while touching both. There is no commit boundary corresponding to one task's lifecycle.

## Deliverables

1. A `just` recipe or pre-push hook implementing the rule.
2. Correction of the stale squash guidance in task-228, which predates the auto-committer.
3. A short `AGENTS.md` note if the recipe is not self-explanatory.

## Non-goals

- Do not disable or defer the auto-committer; losing `.tandem/` state is worse than commit noise.
- Do not rewrite pushed history.
- Do not attempt per-task commit grouping.

## Independence

Unblocked. Does not depend on task-239-4 or task-228.
