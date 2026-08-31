---
id: task-7
type: task
title: "Fix 0.12.0 CLI defects surfaced by the protocol 0.3.0 migration"
priority: "high"
effort: "medium"
references: ["task-3", "task-4"]
tags: ["cli", "protocol", "rules"]
accord:
  status: "claimed"
  assignee: "pi"
  claimedAt: "2026-08-31T21:09:25Z"
  updatedAt: "2026-08-31T21:17:41Z"
createdAt: "2026-08-31T20:53:53Z"
updatedAt: "2026-08-31T21:17:41Z"
archivedAt: "2026-08-31T21:17:41Z"
resolution:
  outcome: "completed"
---
## Completion

Fixed the two migration-surfaced 0.12.0 defects: 'tandem add decision' now writes only to .tandem/decisions/ and stops emitting the manual date field (D46/D41); rules moved to the per-file model - one Markdown file per rule in .tandem/rules/ with composite ids like always-12, category stored in the record, optional source, timestamps, rule text as body; rules add/edit/delete round-trip per-file with edit --clear source; rules list prints composite ids and supports category filtering; TUI Rules view and web rules API read the per-file store. This workspace's 16 config-backed rules were migrated to per-file records and the tandem.md rules block cleared. Verified: 233 unit + 5 process tests, fmt/clippy clean, release build, scratch repro (decision placement, no date, rules round-trip), rendered TUI Rules pane shows category tabs and rows from files.