---
id: task-7
type: task
title: "Fix 0.12.0 CLI defects surfaced by the protocol 0.3.0 migration"
state: todo
priority: "high"
effort: "medium"
references: ["task-2", "task-3", "task-4"]
tags: ["cli", "protocol", "rules"]
accord:
  status: ready
  acceptance: ["tandem update <decision-id> --status accepted works and writes decidedAt (type-aware update, D34)", "tandem add decision writes only to .tandem/decisions/ (D46)", "tandem rules add/edit/delete operate on one file per rule in .tandem/rules/ with composite ids like always-12, and rules list shows composite ids (D46/D49)", "Existing config-backed rules are migrated to per-file records", "full test/clippy/release suite green and a 0.12.x regression release gate passes"]
createdAt: "2026-08-31T20:53:53Z"
updatedAt: "2026-08-31T20:53:53Z"
---

## Description

The semi-manual migration to protocol 0.3.0 surfaced three shipped 0.12.0 defects: decision documents cannot be updated via the CLI (update is task-only), add decision writes to .tandem/tasks instead of .tandem/decisions, and rules remain config-backed with flat ids instead of per-file composite-id records. Scope only these fixes plus migrating the config-backed rules; the Pi adapter and web redesign remain separate.
