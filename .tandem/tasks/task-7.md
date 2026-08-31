---
id: task-7
type: task
title: "Fix 0.12.0 CLI defects surfaced by the protocol 0.3.0 migration"
state: "in-progress"
priority: "high"
effort: "medium"
references: ["task-3", "task-4"]
tags: ["cli", "protocol", "rules"]
accord:
  status: "claimed"
  assignee: "pi"
  claimedAt: "2026-08-31T21:09:25Z"
  updatedAt: "2026-08-31T21:09:25Z"
createdAt: "2026-08-31T20:53:53Z"
updatedAt: "2026-08-31T21:09:25Z"
---
The semi-manual migration to protocol 0.3.0 surfaced two shipped 0.12.0 defects: 'tandem add decision' writes decision documents into .tandem/tasks instead of .tandem/decisions, and rules remain config-backed with flat numeric ids instead of the designed one-file-per-rule composite-id records in .tandem/rules. Scope only these two fixes plus migrating existing config-backed rules into per-file records; the Pi adapter, web redesign, and general update-command behavior remain separate.