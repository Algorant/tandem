---
id: task-41
type: task
title: "Add GitHub Pages deployment workflow for docs site"
state: todo
priority: "medium"
parentId: "task-36"
blockers: ["task-38", "task-39", "task-40"]
references: ["task-24"]
relatedFiles: [".github/workflows/docs.yml", "site/"]
tags: ["docs", "site", "github-pages", "ci"]
createdAt: "2026-06-28T16:58:46Z"
updatedAt: "2026-06-28T16:58:46Z"
---

## Description

Add a GitHub Actions workflow that builds the Astro/Starlight site from site/ and deploys site/dist to GitHub Pages. Keep it compatible with the private repo settings and document any GitHub Pages setup steps that must be completed in repository settings.
