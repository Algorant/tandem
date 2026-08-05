---
id: task-206
type: task
title: "Overhaul Extensions page"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/extensions/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T02:12:08Z"
updatedAt: "2026-08-05T02:56:47Z"
accord:
  status: "accepted"
  assignee: "worker-task-206-f3f9244d"
  claimedAt: "2026-08-05T02:26:11Z"
  deliveredAt: "2026-08-05T02:45:57Z"
  deliverables: ["docs/extensions/index.md"]
  validation:
    commands: ["git diff --check passed", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Approved after Extensions simplification review, source-link correction, build, link validation, and local preview verification."
  filesChanged: ["docs/extensions/index.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:55:50Z"
assignee: "worker-task-206-f3f9244d"
completedAt: "2026-08-05T02:56:47Z"
completion:
  summary: "Shipped the simplified Extensions page."
  filesChanged: ["docs/extensions/index.md"]
  validation: "just site-build; cd site && bun run check:links; just docs reached Astro ready state"
  reviewer: "orchestrator"
---
## Approved Extensions page direction

- Remove the adapter principle, authority layers, integration sequence, and technical boundary material.
- Keep the page simple and welcoming.
- The page should primarily contain one section titled `Official Pi extension`.
- Add a placeholder for the official `pi-tandem` extension, to be filled with its description, setup instructions, and capabilities later.
- Add a link to the `extensions/pi-tandem/` source or its published documentation when available.
- Do not add additional extension listings until there are real official extensions to document.