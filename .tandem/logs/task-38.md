---
id: task-38
type: task
title: "Create Astro Starlight site project"
priority: "high"
parentId: "task-36"
blockers: ["task-37"]
references: ["task-24"]
relatedFiles: ["site/", "site/astro.config.mjs", "site/package.json"]
tags: ["docs", "site", "astro", "starlight"]
createdAt: "2026-06-28T16:58:46Z"
updatedAt: "2026-06-28T17:29:31Z"
accord:
  status: "delivered"
  assignee: "docs-site-36-41"
  claimedAt: "2026-06-28T17:27:14Z"
  deliveredAt: "2026-06-28T17:29:31Z"
  validation:
    commands: ["cd site && npm run build; git diff --check"]
  summary: "Created a minimal Astro Starlight project under site/ with package scripts, config, content collection setup, and placeholder page."
  updatedAt: "2026-06-28T17:29:31Z"
completedAt: "2026-06-28T17:29:31Z"
completion:
  summary: "Created minimal site/ Astro Starlight project."
---

## Description

Create a minimal site/ Astro Starlight project for rendering Tandem documentation. Keep it separate from docs/ so Markdown remains usable as source content and the site owns presentation, navigation, and build tooling.
