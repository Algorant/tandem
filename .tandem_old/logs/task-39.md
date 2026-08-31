---
id: task-39
type: task
title: "Wire docs source into Astro site"
priority: "high"
parentId: "task-36"
blockers: ["task-37", "task-38"]
references: ["task-24"]
relatedFiles: ["docs/", "site/src/content/", "site/astro.config.mjs"]
tags: ["docs", "site", "astro", "markdown"]
createdAt: "2026-06-28T16:58:46Z"
updatedAt: "2026-06-28T17:30:44Z"
accord:
  status: "delivered"
  assignee: "docs-site-36-41"
  claimedAt: "2026-06-28T17:30:14Z"
  deliveredAt: "2026-06-28T17:30:44Z"
  validation:
    commands: ["cd site && npm run build; git diff --check"]
  summary: "Wired repository docs/ into Starlight with a documented sync script and prebuild/predev hooks, avoiding hand-maintained duplicate docs."
  updatedAt: "2026-06-28T17:30:44Z"
completedAt: "2026-06-28T17:30:44Z"
completion:
  summary: "Wired docs/ Markdown source into the Astro Starlight site via sync step."
---

## Description

Connect docs/ Markdown content to the site/ Astro Starlight build. Choose the simplest reliable approach, such as Starlight content config or a small copy/sync step, while avoiding duplicated source-of-truth docs.
