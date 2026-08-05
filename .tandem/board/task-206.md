---
id: task-206
type: task
title: "Overhaul Extensions page"
state: "validation"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/extensions/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T02:12:08Z"
updatedAt: "2026-08-05T02:45:57Z"
accord:
  status: "delivered"
  assignee: "worker-task-206-f3f9244d"
  claimedAt: "2026-08-05T02:26:11Z"
  deliveredAt: "2026-08-05T02:45:57Z"
  deliverables: ["docs/extensions/index.md"]
  validation:
    commands: ["git diff --check passed", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Merged the simplified Extensions page with the official Pi extension placeholder and source link."
  filesChanged: ["docs/extensions/index.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:45:57Z"
assignee: "worker-task-206-f3f9244d"
---
## Approved Extensions page direction

- Remove the adapter principle, authority layers, integration sequence, and technical boundary material.
- Keep the page simple and welcoming.
- The page should primarily contain one section titled `Official Pi extension`.
- Add a placeholder for the official `pi-tandem` extension, to be filled with its description, setup instructions, and capabilities later.
- Add a link to the `extensions/pi-tandem/` source or its published documentation when available.
- Do not add additional extension listings until there are real official extensions to document.