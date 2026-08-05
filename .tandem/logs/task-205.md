---
id: task-205
type: task
title: "Create Fully Agentic workflow page"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/guides/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T02:08:56Z"
updatedAt: "2026-08-05T02:56:43Z"
accord:
  status: "accepted"
  assignee: "worker-task-205-a394f1e9"
  claimedAt: "2026-08-05T02:26:10Z"
  deliveredAt: "2026-08-05T02:45:52Z"
  deliverables: ["docs/guides/fully-agentic-workflow.md"]
  validation:
    commands: ["git diff --check passed", "git diff --check on Worker commit", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Approved after Fully Agentic workflow review, Mermaid validation, build, link validation, and local preview verification."
  filesChanged: ["docs/guides/fully-agentic-workflow.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:55:46Z"
assignee: "worker-task-205-a394f1e9"
completedAt: "2026-08-05T02:56:43Z"
completion:
  summary: "Shipped the Fully Agentic workflow page."
  filesChanged: ["docs/guides/fully-agentic-workflow.md"]
  validation: "just site-build; cd site && bun run check:links; just docs reached Astro ready state"
  reviewer: "orchestrator"
---

## Description

Create the Fully Agentic workflow page under the Workflows section. Show the project owner's workflow using Pi, Herdr, Codex, and the Shep integration layer to delegate tasks to Workers sequentially or in parallel, with delivery evidence and validation. Include a clear diagram and an approachable explanation.
