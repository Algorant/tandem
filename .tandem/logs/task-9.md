---
id: task-9
type: task
title: "Make accord acceptance durable and show output agent-readable"
priority: "high"
effort: "medium"
relatedFiles: ["tandem/src/protocol/accord.rs", "tandem/src/app/accord.rs", "tandem/src/cli/commands.rs"]
tags: ["protocol", "cli", "accord"]
accord:
  status: "accepted"
  acceptance: ["accord.acceptance survives every accord transition", "update writes accord definition fields and reports only real changes", "show --json returns the full record including body, accord, and location", "All three subtasks are completed in order: persistence, repair, then read output"]
  assignee: "pi"
  claimedAt: "2026-09-01T04:42:14Z"
  deliveredAt: "2026-09-01T04:51:32Z"
  summary: "All three subtasks landed"
  evidence: ["244 tests passing"]
  updatedAt: "2026-09-01T04:51:32Z"
createdAt: "2026-09-01T04:33:41Z"
updatedAt: "2026-09-01T04:51:32Z"
archivedAt: "2026-09-01T04:51:32Z"
resolution:
  outcome: "completed"
---

## Description

## Problem

Two defects in the 0.12.x clap cutover, one causing silent data loss and one leaving the agent read path empty.

1. `accord claim` and every later transition rewrite the accord block without `accord.acceptance`, so acceptance criteria are destroyed the moment work starts. `AccordRecord` (`tandem/src/protocol/accord.rs:6`) has no `acceptance` field.
2. `show --json` returns `{id, type, title}` (`tandem/src/cli/commands.rs:139`). No command returns a task body or its accord, so agent consumers have no read path.

## Why these are one Task

The read fix depends on the persistence fix. Restoring `show` output first would faithfully report that every claimed task has no acceptance criteria, which looks like a second bug and is the same one.

## Scope boundary

The TUI is the human read surface and is not being replaced. Human `show` output stays a short identity-and-status block. The requirement is the JSON payload, because agents cannot run the TUI.

## Reference

See `plan/cli-protocol-cutover.md` D17, D20, D31, D53. Neither behavior was removed by decision; both were dropped during the clap rewrite.
