---
id: task-204
type: task
title: "Create Human in the Loop workflow page"
state: "validation"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/guides/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T02:08:56Z"
updatedAt: "2026-08-05T02:45:48Z"
accord:
  status: "delivered"
  assignee: "worker-task-204-23b63f66"
  claimedAt: "2026-08-05T02:26:11Z"
  deliveredAt: "2026-08-05T02:45:48Z"
  deliverables: ["docs/guides/human-in-the-loop.md"]
  validation:
    commands: ["git diff --check passed", "Worker cd site && bun install --frozen-lockfile && bun run check:docs passed", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Merged the Human in the Loop workflow page with Claude/Codex adversarial review and human approval guidance."
  filesChanged: ["docs/guides/human-in-the-loop.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:45:48Z"
assignee: "worker-task-204-23b63f66"
---

## Description

Create the Human in the Loop workflow page under the Workflows section. Show an adversarial workflow where Claude and Codex alternate implementation and review, with a human making the final approval decision. Include a clear diagram and an approachable explanation of the handoff, validation, and feedback loop.
