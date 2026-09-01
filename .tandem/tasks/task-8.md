---
id: task-8
type: task
title: "Sort task IDs numerically instead of lexicographically"
state: todo
priority: "medium"
effort: "small"
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/web.rs", "tandem/src/tui/decisions.rs", "tandem/src/app/queries.rs"]
tags: ["tui", "protocol", "bug"]
accord:
  status: ready
  acceptance: ["Board TODO column orders #1, #2, #3, #4, #5, #10", "CLI list, search, logs, and decisions use the same numeric ordering", "Subtask IDs order task-2-2 before task-2-10", "Unit tests cover the ordering helper, including mixed prefixes and subtask suffixes", "Verified in a rendered Herdr pane, not only by unit test"]
createdAt: "2026-09-01T04:08:46Z"
updatedAt: "2026-09-01T04:08:46Z"
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
