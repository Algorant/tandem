---
id: task-197
type: task
title: "Overhaul home / landing page"
state: "validation"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-04T23:33:39Z"
updatedAt: "2026-08-05T02:45:13Z"
accord:
  status: "delivered"
  assignee: "worker-task-197-4b43e283"
  claimedAt: "2026-08-04T23:38:41Z"
  deliveredAt: "2026-08-05T02:45:13Z"
  deliverables: ["docs/index.md", "site/src/styles/verdigris.css"]
  validation:
    commands: ["just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files", "just docs reached Astro ready state"]
  summary: "Merged landing page implementation with approved hero, CTAs, capability sections, and four-color loop."
  filesChanged: ["docs/index.md", "site/src/styles/verdigris.css"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:45:13Z"
assignee: "worker-task-197-4b43e283"
---
## Approved landing-page direction

The project owner approved the proposed landing-page direction in Sideshow.

### Required content

- Hero positioning: human and agent work in the same repository.
- Primary CTA: Start with the quickstart.
- Secondary CTA: Explore the concepts.
- Capability cards: tasks/workflows, accords/review, decisions/logs.
- Four-step section titled `A simple loop` with distinct colors:
  1. Define the work as tasks.
  2. Agree on acceptance criteria.
  3. Give the work to an agent to complete.
  4. Have another agent review that work or show it to the project owner for final approval.

### Visual direction

- Keep the current Verdigris visual language.
- Use a restrained, documentation-first layout.
- Give each simple-loop step a distinct semantic accent color.
- Preserve responsive behavior and light/dark theme support.

Implementation is approved for delegation. Keep final acceptance in validation until the rendered page is reviewed.