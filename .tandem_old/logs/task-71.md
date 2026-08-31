---
id: task-71
type: task
title: "Fix docs deploy workflow runtime and package-manager grounding"
priority: "high"
blockers: ["task-70"]
relatedFiles: [".github/workflows/docs.yml", "site/package.json", "site/package-lock.json", "justfile"]
tags: ["docs", "ci", "deployment", "bun", "node"]
createdAt: "2026-06-30T12:02:43Z"
updatedAt: "2026-07-01T03:10:21Z"
subtasks:
  - id: task-71-1
    title: "Update workflow runtime/package-manager setup"
    completed: false
  - id: task-71-2
    title: "Align local docs commands and lockfiles with chosen package manager"
    completed: false
  - id: task-71-3
    title: "Run docs build validation locally and verify GitHub Pages deploy succeeds"
    completed: false
  - id: task-71-4
    title: "Document rationale for selected runtime/package-manager"
    completed: false
accord:
  status: "accepted"
  deliveredAt: "2026-07-01T03:10:03Z"
  deliverables: [".github/workflows/docs.yml uses node-version-file: site/.node-version with npm cache/lockfile.", "site/.node-version added with 24; site/package.json and lock include engines.node >=22.12.0.", "justfile docs shortcuts check Node major and site-build mirrors CI with npm ci && npm run build.", "Docs updated in docs/guides/docs-site.md and site/README.md."]
  validation:
    commands: ["Parent ran git diff --check: passed.", "Parent ran mise x node@24 -- just site-build: passed; npm ci and Astro build completed, existing non-blocking Starlight 404 warning observed."]
  summary: "Accepted: docs deploy/runtime changes are scoped, grounded in site/.node-version + package-lock/npm ci, and passed parent validation build."
  evidence: ["git diff -- .github/workflows/docs.yml justfile site/package.json site/package-lock.json site/.node-version docs/guides/docs-site.md site/README.md", "mise x node@24 -- just site-build"]
  filesChanged: [".github/workflows/docs.yml", "site/.node-version", "site/package.json", "site/package-lock.json", "justfile", "docs/guides/docs-site.md", "site/README.md"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-07-01T03:10:13Z"
completedAt: "2026-07-01T03:10:21Z"
completion:
  summary: "Completed docs deploy runtime/package-manager grounding. Workflow now reads site/.node-version, local just docs shortcuts enforce Node 24 and mirror CI with npm ci, package engines document Astro's Node floor, and docs explain npm/package-lock policy."
  validation: "git diff --check passed; mise x node@24 -- just site-build passed with npm ci and Astro build completing."
  reviewer: "pi-orchestrator"
---

## Description

After the docs runtime investigation is complete, update the docs deploy workflow and local shortcuts so the GitHub Pages build uses a supported, explicitly justified runtime/package manager. Prefer Bun if investigation confirms it is appropriate for this docs site; otherwise use a grounded stable Node version that satisfies Astro/Starlight. Ensure local and CI commands stay aligned and avoid arbitrary version pins.
