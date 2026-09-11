---
id: task-3
type: task
title: "CLI: add decision writes to .tandem/tasks"
state: "in-progress"
priority: "low"
tags: ["papercut"]
accord:
  status: "claimed"
  acceptance: ["decisions/ is the only runtime path for decision documents (D46)"]
  claimedAt: "2026-09-11T14:22:41Z"
  updatedAt: "2026-09-11T14:22:41Z"
createdAt: "2026-08-31T20:53:43Z"
updatedAt: "2026-09-11T14:22:41Z"
assignee: "pi-orchestrator"
---

## Description

Migrating surfaced this: 'tandem add decision' wrote decision-N.md into .tandem/tasks/ instead of .tandem/decisions/. Reader scans both, so nothing visibly broke, but the layout contract is violated and dir-based tooling breaks.
