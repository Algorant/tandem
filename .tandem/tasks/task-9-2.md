---
id: task-9-2
type: task
title: "Return the full record from show --json"
state: todo
priority: "high"
effort: "small"
parentId: "task-9"
blockers: ["task-9-1"]
relatedFiles: ["tandem/src/cli/commands.rs", "tandem/src/web.rs"]
tags: ["cli", "protocol"]
accord:
  status: ready
  acceptance: ["show --json returns frontmatter and body: kind, state, priority, effort, assignee, dueDate, parentId, parent relationship, tags, blockers, references, relatedFiles, the whole accord object, validation, timestamps, decision metadata for decisions, resolution for logs", "The JSON projection is shared with the web read models rather than written a second time", "Human show output remains a short identity-and-status block and does not replicate the TUI", "A test asserts a claimed task returns its acceptance criteria through show --json"]
createdAt: "2026-09-01T04:34:00Z"
updatedAt: "2026-09-01T04:34:00Z"
---

## Description

## Problem

`tandem/src/cli/commands.rs:139` hardcodes `{id, type, title}`. The record layer holds everything: the TUI and web read models get full documents from the same `app`/`project` path. Only the CLI projection was dropped in the clap rewrite.

## Why this matters

The CLI is the agent interface. Agents cannot run the TUI, so `show --json` is their only read path. Worker delegation carries the task body into the prompt and no command returns a body.

This is not a human-ergonomics fix. The TUI is the human read surface and stays that way.

## Direction

`web.rs` already has `DocumentSummaryDto` and `DocumentDetailDto` over the same `app` layer with exactly these fields. Share that projection; do not author a second one.

## Deliberately out of scope

`list` and `search` row enrichment. An agent that can read one full record can ask for what it needs. Revisit only with observed demand.

## Not deliberate removals

D31 describes show as rendering by type, D53 makes `--json` universal, and the 0.12.0 removal list names only commands and flags. No decision shrank read output.
