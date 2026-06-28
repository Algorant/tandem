---
id: task-20
type: task
title: "Surface Board accord details for selected tasks"
state: todo
priority: "high"
references: ["task-5", "task-11"]
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/plan/spec.md"]
tags: ["ui", "tui", "board", "accord"]
createdAt: "2026-06-28T00:17:02Z"
updatedAt: "2026-06-28T00:17:02Z"
accord:
  status: "ready"
  deliverables: ["code:tandem-tui/src/tui.rs:Board accord detail rendering", "docs:tandem-tui/plan/spec.md:Board accord detail behavior"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check && cargo test && cargo build", "PTY/manual smoke selecting tasks with ready/claimed/delivered accords"]
  constraints: ["Do not change protocol accord semantics."]
  updatedAt: "2026-06-28T00:17:02Z"
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
