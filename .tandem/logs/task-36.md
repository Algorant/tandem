---
id: task-36
type: task
title: "Implement Tandem docs site foundation"
priority: "high"
references: ["task-24"]
relatedFiles: ["docs", "site", ".github/workflows"]
tags: ["docs", "site", "astro", "github-pages"]
createdAt: "2026-06-28T16:58:46Z"
updatedAt: "2026-06-28T17:34:29Z"
subtasks:
  - id: task-36-1
    title: "Create the docs/ content skeleton"
    completed: false
  - id: task-36-2
    title: "Create the site/ Astro Starlight project"
    completed: false
  - id: task-36-3
    title: "Wire docs/ Markdown content into the site build"
    completed: false
  - id: task-36-4
    title: "Document local preview and build commands"
    completed: false
  - id: task-36-5
    title: "Add GitHub Pages deployment workflow"
    completed: false
accord:
  status: "delivered"
  assignee: "docs-site-36-41"
  claimedAt: "2026-06-28T17:26:15Z"
  deliveredAt: "2026-06-28T17:34:29Z"
  validation:
    commands: ["cd site && npm ci && npm run build && npm audit --audit-level=high; git diff --check"]
  summary: "Implemented docs site foundation: canonical docs/ Markdown skeleton, site/ Astro Starlight project, docs sync wiring, local workflow docs, GitHub Pages workflow, and dependency audit cleanup."
  updatedAt: "2026-06-28T17:34:29Z"
completedAt: "2026-06-28T17:34:29Z"
completion:
  summary: "Implemented Tandem docs site foundation across docs/, site/, and GitHub Pages workflow."
---

## Description

Implement the accepted docs platform direction from task-24: keep canonical Markdown docs in docs/, build a separate Astro/Starlight site in site/, and deploy static output through GitHub Pages. Keep the first slice minimal and maintainable.
