---
id: task-40
type: task
title: "Document docs local preview and build workflow"
priority: "medium"
parentId: "task-36"
blockers: ["task-38", "task-39"]
references: ["task-24"]
relatedFiles: ["README.md", "docs/README.md", "site/package.json"]
tags: ["docs", "site", "developer-experience"]
createdAt: "2026-06-28T16:58:46Z"
updatedAt: "2026-06-28T17:31:31Z"
accord:
  status: "delivered"
  assignee: "docs-site-36-41"
  claimedAt: "2026-06-28T17:30:58Z"
  deliveredAt: "2026-06-28T17:31:31Z"
  validation:
    commands: ["cd site && npm run build; git diff --check"]
  summary: "Documented local docs install, preview, sync, and build workflow in site README and docs guide."
  updatedAt: "2026-06-28T17:31:31Z"
completedAt: "2026-06-28T17:31:31Z"
completion:
  summary: "Documented docs local preview and build workflow."
---

## Description

Document the local docs workflow: install dependencies, preview the Astro/Starlight site, build static output, and explain that docs/ is the Markdown source while site/ owns rendering.
