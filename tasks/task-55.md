---
id: task-55
uid: 73825776-f51e-40df-a7b1-9ee17ae8bc2e
type: task
title: "Add research and papercut as first-class Task kinds"
state: "in-progress"
priority: "medium"
effort: "large"
references: ["task-22", "decision-5"]
relatedFiles: ["protocol/README.md", "tandem/src/protocol/document.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/papercuts.rs"]
tags: ["protocol", "taxonomy"]
accord:
  status: "claimed"
  acceptance: ["`research` and `papercut` are valid Task kinds across protocol, CLI, TUI, and docs, with the agreed required fields, default priority, and placement rule enforced by Tandem.", "Active Board records tagged `research` or `papercut` migrate to the matching kind. Archived Logs are untouched.", "No Tandem code path classifies research or papercut work by tag after this change."]
  claimedAt: "2026-10-01T22:43:11Z"
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-01T22:43:11Z"
createdAt: "2026-10-01T22:20:35Z"
updatedAt: "2026-10-01T22:43:11Z"
assignee: "worker-task-55-e2f38237"
---

## Description

## Context
Direction agreed with Algorant on 2026-10-01 in a ~/.pi Pi conversation. This answers task-22's modeling question. Sideshow mockups and the full discussion trail: http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8

## Decided
- `kind` values: `epic`, `research`, `papercut`. A Task with no kind is a standard Task. Tags go back to being topical only.
- No standing Epics or container records. Classification comes from kind alone.
- Required at `add` (deliberately tiny): papercut needs only a title, with acceptance optional; task and research keep title + acceptance.
- `kind: papercut` defaults to `priority: low` (can be overridden).
- Papercut placement: top level or under an Epic, **never a Subtask**. Research can sit anywhere.
- Archived Logs stay as they are and keep their legacy `research`/`papercut` tags. No code reads those tags as kinds.
- No tag fallback: everything keyed on the `research`/`papercut` tags switches to kind in this change.
- The TUI stays static; lanes belong in `tandem web` (separate Task).

## Out of scope (owned by ~/.pi)
- Intake triage rubric and follow-up questions
- The intake verdict (an `## Intake` body section written by Pi)
- Kind-specific Worker/Subagent rules
- Running all Tasks of one kind

