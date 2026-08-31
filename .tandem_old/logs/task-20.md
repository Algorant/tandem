---
id: task-20
type: task
title: "Surface Board accord details for selected tasks"
priority: "high"
references: ["task-5", "task-11"]
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/plan/spec.md"]
tags: ["ui", "tui", "board", "accord"]
createdAt: "2026-06-28T00:17:02Z"
updatedAt: "2026-06-28T02:04:09Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-06-28T01:43:16Z"
  deliveredAt: "2026-06-28T02:04:09Z"
  deliverables: ["code:tandem-tui/src/tui.rs:Board accord detail rendering", "docs:tandem-tui/plan/spec.md:Board accord detail behavior"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check && cargo test && cargo build"]
  constraints: ["Do not change protocol accord semantics."]
  summary: "Added Board selected-task accord details and read-only next-action hints."
  evidence: ["Merged 93e1fd8 via 7ff3e75; integrated validation passed."]
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/plan/spec.md"]
  reviewer: "pi"
  updatedAt: "2026-06-28T02:04:09Z"
completedAt: "2026-06-28T02:04:09Z"
completion:
  summary: "Integrated Board accord detail rendering with status styling, metadata, and CLI hints."
  validation: "cargo fmt --check; cargo test; cargo build"
---

## Description

Make accord details visible and useful from the Board pane for selected tasks.

Context:
- Board rows can show accord status, but there is no obvious dedicated place to inspect full accord details from Board.
- Agents and humans need to see ready/claimed/delivered context, deliverables, validation commands, evidence, files changed, reviewer/note/reason, and next actions without leaving the Board.

Acceptance direction:
- Add a clear Board detail section or subpane area for accord metadata on the selected task.
- Preserve task body/details and make accord content discoverable without overwhelming the list rows.
- Surface action hints for the next likely CLI/TUI accord actions, but do not implement mutations unless explicitly in scope.
- Ensure delivered/accepted/rework/blocked states are visually distinct and documented.
