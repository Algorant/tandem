---
id: task-9-2
type: task
title: "Return the full record from show --json"
priority: "high"
effort: "small"
parentId: "task-9"
blockers: ["task-9-1"]
relatedFiles: ["tandem/src/cli/commands.rs", "tandem/src/web.rs"]
tags: ["cli", "protocol"]
accord:
  status: "accepted"
  acceptance: ["show --json returns frontmatter and body: kind, state, priority, effort, assignee, dueDate, parentId, parent relationship, tags, blockers, references, relatedFiles, the whole accord object, validation, timestamps, decision metadata for decisions, resolution for logs", "The JSON projection is shared with the web read models rather than written a second time", "Human show output remains a short identity-and-status block and does not replicate the TUI", "A test asserts a claimed task returns its acceptance criteria through show --json"]
  assignee: "pi"
  claimedAt: "2026-09-01T04:51:29Z"
  deliveredAt: "2026-09-01T04:51:29Z"
  summary: "implemented and verified"
  evidence: ["cargo test: 244 passing", "rendered Herdr pane check"]
  updatedAt: "2026-09-01T04:51:29Z"
createdAt: "2026-09-01T04:34:00Z"
updatedAt: "2026-09-01T04:51:29Z"
archivedAt: "2026-09-01T04:51:29Z"
resolution:
  outcome: "completed"
---
## Problem

`tandem/src/cli/commands.rs:139` hardcodes `{id, type, title}`. The record layer holds everything: the TUI and web read models get full documents from the same `app`/`project` path. Only the CLI projection was dropped in the clap rewrite.

## Why this matters

The CLI is the agent interface. Agents cannot run the TUI, so `show --json` is their only read path. Worker delegation carries the task body into the prompt and no command returns a body.

This is not a human-ergonomics fix. The TUI is the human read surface and stays that way.

## Required: location on every document

`show --json` must always return `location` (`board` | `logs`) for every document type. It is the unambiguous active-vs-archived signal and must never be inferred from `resolution`.

`resolution` cannot serve that role. `web.rs:697` constructs it only when the document is in Logs and is a task, so an archived decision has none, and `protocol/workflow.rs:94` defaults `resolution_outcome` to `completed` when the field is absent, making its presence derived rather than stored.

`DocumentLocation` (`protocol/hierarchy.rs:59`) is exactly `board` | `logs` and is derived from storage, not content, so it cannot be absent. Note it serializes as `board` even though 0.3.0 stores active tasks in `.tandem/tasks/`.

Confirmed with the Pi adapter: both `worker_start` and `worker_recover` guards will be rewritten against `location === "logs"`, replacing a stale 0.2 `state === "completed" || "canceled"` test.

## Additional acceptance criteria

These belong in the accord but cannot be written while task-9-3 is open, because `update --acceptance` silently discards its input:

- `show --json` always returns `location` for every document type, never omitted, never inferred from `resolution`.
- A test asserts `location` is present for an active task, an archived task, and a decision.

## Direction

`web.rs` already has `DocumentSummaryDto` and `DocumentDetailDto` over the same `app` layer with exactly these fields. Share that projection; do not author a second one.

## Deliberately out of scope

`list` and `search` row enrichment. Confirmed with the Pi adapter: no Pi call site consumes list rows beyond `id` and `title`, and every filter is passed server-side. Children are covered by `list --parent`, so `subtasks` in `show` was withdrawn. Revisit only with observed demand.

## Not deliberate removals

D31 describes show as rendering by type, D53 makes `--json` universal, and the 0.12.0 removal list names only commands and flags. No decision shrank read output.