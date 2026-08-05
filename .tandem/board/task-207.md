---
id: task-207
type: task
title: "Overhaul Skills page"
state: "validation"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/skills/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T02:18:01Z"
updatedAt: "2026-08-05T02:46:05Z"
accord:
  status: "delivered"
  assignee: "worker-task-207-64ed7833"
  claimedAt: "2026-08-05T02:26:11Z"
  deliveredAt: "2026-08-05T02:46:05Z"
  deliverables: ["docs/skills/index.md"]
  validation:
    commands: ["git diff --check passed", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Merged the Skills page placeholders for generic agent guidance, Codex Skill, and Claude Code Skill."
  filesChanged: ["docs/skills/index.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:46:05Z"
assignee: "worker-task-207-64ed7833"
---

## Description

Review and define the Skills page for trytandem.dev. Keep it simple with placeholders for generic agent guidance, a Codex Skill, and a Claude Code Skill. Add real descriptions and links later as each skill is published.
