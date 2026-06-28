---
id: task-22
type: task
title: "Define Tandem task tag taxonomy for delegation"
state: todo
priority: "medium"
references: ["task-12", "task-13"]
relatedFiles: ["plan/todo.md", "AGENTS.md"]
tags: ["docs", "rules", "taxonomy", "delegation"]
createdAt: "2026-06-28T00:17:03Z"
updatedAt: "2026-06-28T02:22:39Z"
accord:
  status: "claimed"
  assignee: "pi"
  claimedAt: "2026-06-28T02:18:41Z"
  deliverables: ["docs:plan/todo.md:tag taxonomy or convention notes", "docs:AGENTS.md:agent task tagging guidance if needed"]
  validation:
    commands: ["./tandem-tui/target/debug/tdm list --tag pi-tandem", "git diff --check"]
  constraints: ["Prefer convention and docs before protocol/schema changes."]
  updatedAt: "2026-06-28T02:18:41Z"
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
