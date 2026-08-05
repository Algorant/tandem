---
id: task-204
type: task
title: "Create Human in the Loop workflow page"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/guides/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T02:08:56Z"
updatedAt: "2026-08-05T02:56:39Z"
accord:
  status: "accepted"
  assignee: "worker-task-204-23b63f66"
  claimedAt: "2026-08-05T02:26:11Z"
  deliveredAt: "2026-08-05T02:45:48Z"
  deliverables: ["docs/guides/human-in-the-loop.md"]
  validation:
    commands: ["git diff --check passed", "Worker cd site && bun install --frozen-lockfile && bun run check:docs passed", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Approved after Human in the Loop workflow review, Mermaid validation, build, link validation, and local preview verification."
  filesChanged: ["docs/guides/human-in-the-loop.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:55:41Z"
assignee: "worker-task-204-23b63f66"
completedAt: "2026-08-05T02:56:39Z"
completion:
  summary: "Shipped the Human in the Loop workflow page."
  filesChanged: ["docs/guides/human-in-the-loop.md"]
  validation: "just site-build; cd site && bun run check:links; just docs reached Astro ready state"
  reviewer: "orchestrator"
---

## Description

Create the Human in the Loop workflow page under the Workflows section. Show an adversarial workflow where Claude and Codex alternate implementation and review, with a human making the final approval decision. Include a clear diagram and an approachable explanation of the handoff, validation, and feedback loop.
