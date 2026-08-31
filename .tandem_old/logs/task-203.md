---
id: task-203
type: task
title: "Create Workflows section"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/guides/agents-and-adapters.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T02:01:03Z"
updatedAt: "2026-08-05T02:56:35Z"
accord:
  status: "accepted"
  assignee: "worker-task-203-d9291037"
  claimedAt: "2026-08-05T02:41:45Z"
  deliveredAt: "2026-08-05T02:45:41Z"
  deliverables: ["site/astro.config.mjs", "docs/guides/index.md"]
  validation:
    commands: ["git diff --check passed", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Approved after final Workflows, Overview, and Integrations navigation review and combined site validation."
  filesChanged: ["site/astro.config.mjs", "docs/guides/index.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:55:38Z"
assignee: "worker-task-203-d9291037"
completedAt: "2026-08-05T02:56:35Z"
completion:
  summary: "Shipped the Workflows and Integrations navigation overhaul."
  filesChanged: ["site/astro.config.mjs", "docs/guides/index.md"]
  validation: "just site-build; cd site && bun run check:links; just docs reached Astro ready state"
  reviewer: "orchestrator"
---
## Approved Workflows navigation

The project owner approved the Workflows section direction in Sideshow.

- Replace the current Workflows subpage collection with a `Workflows` sidebar section.
- The section contains two separate pages:
  1. `Human in the Loop`
  2. `Fully Agentic`
- Do not use one combined Workflows page for the two examples.
- Keep the existing `/guides/` route available as the section landing route if needed, but make the two workflow pages the primary entries.
- Keep the Integrations section separate.
- Use welcoming plain language and clear diagrams on both pages.