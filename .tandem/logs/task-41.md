---
id: task-41
type: task
title: "Add GitHub Pages deployment workflow for docs site"
priority: "medium"
parentId: "task-36"
blockers: ["task-38", "task-39", "task-40"]
references: ["task-24"]
relatedFiles: [".github/workflows/docs.yml", "site/"]
tags: ["docs", "site", "github-pages", "ci"]
createdAt: "2026-06-28T16:58:46Z"
updatedAt: "2026-06-28T17:32:31Z"
accord:
  status: "delivered"
  assignee: "docs-site-36-41"
  claimedAt: "2026-06-28T17:32:11Z"
  deliveredAt: "2026-06-28T17:32:31Z"
  validation:
    commands: ["cd site && npm run build; git diff --check"]
  summary: "Added GitHub Actions workflow to build site/ and deploy site/dist through GitHub Pages, plus setup documentation."
  updatedAt: "2026-06-28T17:32:31Z"
completedAt: "2026-06-28T17:32:31Z"
completion:
  summary: "Added GitHub Pages deployment workflow for docs site."
---

## Description

Add a GitHub Actions workflow that builds the Astro/Starlight site from site/ and deploys site/dist to GitHub Pages. Keep it compatible with the private repo settings and document any GitHub Pages setup steps that must be completed in repository settings.
