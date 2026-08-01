---
id: task-22
type: task
title: "Define Tandem task tag taxonomy for delegation"
priority: "medium"
references: ["task-12", "task-13"]
relatedFiles: ["plan/todo.md", "AGENTS.md"]
tags: ["docs", "rules", "taxonomy", "delegation"]
createdAt: "2026-06-28T00:17:03Z"
updatedAt: "2026-06-28T02:39:55Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-06-28T02:18:41Z"
  deliveredAt: "2026-06-28T02:39:55Z"
  deliverables: ["docs:plan/todo.md:tag taxonomy or convention notes", "docs:AGENTS.md:agent task tagging guidance if needed"]
  validation:
    commands: ["tdm list --tag pi-tandem; tdm list --tag tui; tdm rules list"]
  constraints: ["Prefer convention and docs before protocol/schema changes."]
  summary: "Defined lightweight Tandem tag taxonomy and applied it to active tasks while preserving task-9 subtask split."
  evidence: ["Merged 4463afd via 49f68a8 with task-9 metadata reconciliation; integrated validation passed."]
  filesChanged: [".tandem/tandem.md", "plan/todo.md", ".tandem/board/task-9.md"]
  reviewer: "pi"
  updatedAt: "2026-06-28T02:39:55Z"
completedAt: "2026-06-28T02:39:55Z"
completion:
  summary: "Integrated tag taxonomy rules/conventions and reconciled task-9 tags with the new subtask breakdown."
  validation: "tdm list --tag pi-tandem; tdm list --tag tui; tdm rules list; git diff --check"
---

## Description

Define a lightweight Tandem task tag taxonomy so work can be filtered and delegated by area.

Context:
- As Tandem grows, tasks should be easy to filter into areas such as `ui`, `pi-tandem`, `protocol`, `docs`, `theme`, `keyboard`, `accord`, and `editor`.
- This should remain lightweight and convention-based unless the protocol later needs first-class typed areas.

Acceptance direction:
- Document recommended tags/area conventions in planning docs or rules.
- Apply the convention to current active tasks where useful.
- Keep tags simple enough for `tdm list/search --tag` and future Pi tools to use.
- Avoid changing protocol schema unless a concrete need emerges.
