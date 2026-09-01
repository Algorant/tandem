---
id: task-8
type: task
title: "Sort task IDs numerically instead of lexicographically"
priority: "medium"
effort: "small"
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/web.rs", "tandem/src/tui/decisions.rs", "tandem/src/app/queries.rs"]
tags: ["tui", "protocol", "bug"]
accord:
  status: "ready"
  updatedAt: "2026-09-01T04:15:47Z"
createdAt: "2026-09-01T04:08:46Z"
updatedAt: "2026-09-01T04:15:47Z"
archivedAt: "2026-09-01T04:15:47Z"
resolution:
  outcome: "completed"
---

## Description

## Problem

Documents are ordered by raw string ID comparison, so `task-10` sorts between `task-1` and `task-2`. The TUI Board TODO column currently shows: #1, #10, #2, #3, #4, #5.

## Cause

Every ordering site compares `a.id().cmp(b.id())` as a string:

- `tandem/src/tui/mod.rs:89` `sort_documents`
- `tandem/src/web.rs:644` `sort_documents`
- `tandem/src/web.rs:427` decisions
- `tandem/src/tui/decisions.rs:350` `sorted_decision_docs`
- `tandem/src/app/queries.rs:301` search results
- `tandem/src/app/queries.rs:250` children ordering (verify)

## Fix

Add one shared ID ordering helper that compares the numeric segments of Tandem IDs (`task-N`, `task-N-M`, `decision-N`) as integers with the prefix as the primary key, and route all sort sites through it. ID shape is protocol knowledge, so the helper belongs in `protocol`, not in `tui`/`web`.
