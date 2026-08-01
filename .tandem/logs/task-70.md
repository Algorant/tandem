---
id: task-70
type: task
title: "Investigate docs deploy runtime pinning and package-manager choice"
priority: "high"
relatedFiles: [".github/workflows/docs.yml", "site/package.json", "site/package-lock.json", "justfile"]
tags: ["docs", "ci", "deployment", "bun", "node"]
createdAt: "2026-06-30T12:02:34Z"
updatedAt: "2026-06-30T19:38:46Z"
subtasks:
  - id: task-70-1
    title: "Identify what introduced or maintains the Node 20 pin in docs workflow"
    completed: false
  - id: task-70-2
    title: "Check Astro/Starlight supported Node versions and current stable runtime guidance"
    completed: false
  - id: task-70-3
    title: "Evaluate Bun support for install/build/audit-equivalent needs in GitHub Actions"
    completed: false
  - id: task-70-4
    title: "Recommend whether to use Bun, Node LTS/current, or both"
    completed: false
accord:
  status: "accepted"
  assignee: "shep:task-70-docs-runtime"
  claimedAt: "2026-06-30T19:21:17Z"
  deliveredAt: "2026-06-30T19:27:40Z"
  deliverables: ["Documented runtime/package-manager policy in docs/guides/docs-site.md.", "Recommended changing GitHub Pages docs workflow from Node 20 to Node 24.", "Recommended keeping npm/package-lock/npm ci for docs site for now; defer Bun migration unless intentional."]
  validation:
    commands: ["cd site && npm run build passed", "git diff --check passed"]
  summary: "Accepted objective docs runtime/package-manager research after documentation update and docs build validation passed."
  evidence: ["Commit 5d1f056 documents Node/npm runtime policy and recommends Node 24 for docs deploy."]
  filesChanged: ["docs/guides/docs-site.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-06-30T19:38:42Z"
completedAt: "2026-06-30T19:38:46Z"
completion:
  summary: "Documented docs-site runtime and package-manager policy, recommending Node 24 LTS for deployment and keeping npm/package-lock/npm ci for now."
  validation: "cd site && npm run build passed; git diff --check passed; committed as 5d1f056."
  reviewer: "orchestrator"
---

## Description

Investigate why the GitHub Pages docs workflow pins Node 20 while the current Astro/Starlight toolchain requires >=22.12.0. Determine whether the project should standardize docs-site tasks on Bun instead of npm, and ground any selected runtime/package-manager version in upstream stable/support policy rather than an arbitrary pin. Capture findings and recommended workflow changes before implementation.
