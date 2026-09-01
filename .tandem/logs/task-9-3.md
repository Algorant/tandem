---
id: task-9-3
type: task
title: "update silently drops accord fields and reports success"
priority: "high"
effort: "small"
parentId: "task-9"
relatedFiles: ["tandem/src/app/tasks.rs", "tandem/src/cli/commands.rs"]
tags: ["cli", "accord"]
accord:
  status: "accepted"
  acceptance: ["update --acceptance, --constraint, and --validation write their values to the accord block", "An update that applies no changes does not print Updated <id>; it reports that nothing changed", "Human output and the JSON changes array agree in every case", "A test asserts each accord flag round-trips through update and appears in the file"]
  assignee: "pi"
  claimedAt: "2026-09-01T04:51:29Z"
  deliveredAt: "2026-09-01T04:51:29Z"
  summary: "implemented and verified"
  evidence: ["cargo test: 244 passing", "rendered Herdr pane check"]
  updatedAt: "2026-09-01T04:51:29Z"
createdAt: "2026-09-01T04:38:13Z"
updatedAt: "2026-09-01T04:51:29Z"
archivedAt: "2026-09-01T04:51:29Z"
resolution:
  outcome: "completed"
---

## Description

## Reproduction

```
tandem add task alpha --acceptance "one"
tandem update task-1 --acceptance "two" --acceptance "three"
# prints: Updated task-1
# file still holds: acceptance: ["one"]

tandem update task-1 --constraint c1 --validation v1
# prints: Updated task-1
# file gains neither

tandem update task-1 --acceptance zzz -j
# {"data":{"changes":[],"id":"task-1"},"ok":true,"warnings":[]}
```

`--title` and `--priority` work, so the update path itself is fine. The three accord-nested flags are parsed, accepted, and discarded.

## Two defects

1. `update` cannot write `accord.acceptance`, `accord.constraints`, or `accord.validations` despite advertising the flags in `--help`.
2. Human output prints `Updated <id>` when zero changes were applied. The JSON envelope is honest and returns `changes: []`, so the two surfaces disagree. A caller reading human output is told a write succeeded that did not happen.

## Relationship to task-9-1

task-9-1 stops acceptance criteria being destroyed on claim. This stops them being unrepairable afterward: today `update --acceptance` is the only offered recovery and it silently does nothing. Both are needed for acceptance criteria to be durable in practice.

## Direction

Fix the write path in `app/tasks.rs` so accord list fields follow D36 deterministic replacement like every other list field. Then make the human success message a function of the applied change set rather than of reaching the end of the command.
