---
id: task-200
type: task
title: "Overhaul Concepts page"
state: "validation"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/concepts/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T01:32:21Z"
updatedAt: "2026-08-05T02:45:28Z"
accord:
  status: "delivered"
  assignee: "worker-task-200-5419293e"
  claimedAt: "2026-08-05T02:26:09Z"
  deliveredAt: "2026-08-05T02:45:28Z"
  deliverables: ["docs/concepts/index.md"]
  validation:
    commands: ["git diff --check passed", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Merged the Concepts orientation, lifecycle, hierarchy, Accords, Rules, Decisions, Logs, and Workspace guidance."
  filesChanged: ["docs/concepts/index.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:45:28Z"
assignee: "worker-task-200-5419293e"
---
## Approved Concepts page direction

- Use the heading `A few questions Tandem keeps visible`, not `Three questions`.
- Include four questions:
  - What needs to happen? Tasks, Epics, and Subtasks describe and organize work.
  - Who agreed to do it? Accords make ownership, delivery, validation, and acceptance explicit.
  - How should it be done? Rules provide workspace coordination expectations.
  - What happened? Decisions, Logs, and events preserve project history.
- Explain the active task lifecycle before introducing protocol details.
- Show the Epic → Task → Subtask relationship with plain language.
- After `How work is organized`, add dedicated sections for `Accords`, `Rules`, and `Decisions`.
- Each dedicated section should use brief descriptions or bullet points explaining what the concept does and why it matters.
- Do not include a `Continue exploring` section.
- Keep detailed reference material below the orientation layer.
- Link clearly to Workspace, Quickstart, CLI Reference, TUI, and agent guidance where links are useful in the body.

Implementation remains pending until the page review is complete.