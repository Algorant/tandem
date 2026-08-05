---
id: task-205
type: task
title: "Create Fully Agentic workflow page"
state: "validation"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/guides/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T02:08:56Z"
updatedAt: "2026-08-05T02:45:52Z"
accord:
  status: "delivered"
  assignee: "worker-task-205-a394f1e9"
  claimedAt: "2026-08-05T02:26:10Z"
  deliveredAt: "2026-08-05T02:45:52Z"
  deliverables: ["docs/guides/fully-agentic-workflow.md"]
  validation:
    commands: ["git diff --check passed", "git diff --check on Worker commit", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Merged the Fully Agentic workflow page with Pi, Herdr, Shep, Codex, sequential/parallel delegation, evidence, and validation guidance."
  filesChanged: ["docs/guides/fully-agentic-workflow.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:45:52Z"
assignee: "worker-task-205-a394f1e9"
---

## Description

Create the Fully Agentic workflow page under the Workflows section. Show the project owner's workflow using Pi, Herdr, Codex, and the Shep integration layer to delegate tasks to Workers sequentially or in parallel, with delivery evidence and validation. Include a clear diagram and an approachable explanation.
