---
id: task-39
type: task
title: "Wire docs source into Astro site"
state: todo
priority: "high"
parentId: "task-36"
blockers: ["task-37", "task-38"]
references: ["task-24"]
relatedFiles: ["docs/", "site/src/content/", "site/astro.config.mjs"]
tags: ["docs", "site", "astro", "markdown"]
createdAt: "2026-06-28T16:58:46Z"
updatedAt: "2026-06-28T16:58:46Z"
---

## Description

Connect docs/ Markdown content to the site/ Astro Starlight build. Choose the simplest reliable approach, such as Starlight content config or a small copy/sync step, while avoiding duplicated source-of-truth docs.
