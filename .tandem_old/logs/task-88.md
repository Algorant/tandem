---
id: task-88
type: task
title: "Update docs deploy workflow actions for Node 24 runtime"
priority: "medium"
references: ["task-66"]
relatedFiles: [".github/workflows/docs.yml", "site/.node-version", "site/package.json"]
tags: ["docs", "deployment", "github-actions", "node24"]
createdAt: "2026-07-04T11:46:30Z"
updatedAt: "2026-07-04T11:53:57Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-04T11:51:06Z"
  deliveredAt: "2026-07-04T11:53:48Z"
  deliverables: ["Updated .github/workflows/docs.yml from actions/checkout@v4 to @v7, setup-node@v4 to @v6, upload-pages-artifact@v3 to @v5, and deploy-pages@v4 to @v5.", "Reviewed diff: only .github/workflows/docs.yml changed."]
  validation:
    commands: ["Parent reran cd site && npm run build: passed, 11 pages built.", "Worker reported YAML parsed successfully with Python/PyYAML and cd site && npm ci && npm run build passed.", "Verified latest action metadata earlier: checkout@v7, setup-node@v6, deploy-pages@v5, and upload-artifact used by upload-pages-artifact@v5 declare node24."]
  summary: "PASS: Objective non-visual workflow maintenance task verified. The only code change updates GitHub Actions refs to Node 24-compatible versions, and local docs build passes."
  evidence: ["git diff -- .github/workflows/docs.yml shows four action version updates only.", "git status --short shows only .github/workflows/docs.yml modified."]
  filesChanged: [".github/workflows/docs.yml"]
  reviewer: "orchestrator"
  updatedAt: "2026-07-04T11:53:52Z"
completedAt: "2026-07-04T11:53:57Z"
completion:
  summary: "Updated docs deploy workflow actions to Node 24-compatible versions after verification."
  filesChanged: [".github/workflows/docs.yml"]
  validation: "PASS: Verified diff only changes .github/workflows/docs.yml action refs: checkout@v7, setup-node@v6, upload-pages-artifact@v5, deploy-pages@v5. Parent reran `cd site && npm run build`, which passed with 11 pages built. Worker reported YAML parse and `cd site && npm ci && npm run build` passed. Remaining confirmation is a future pushed GitHub Actions run showing the deprecation warning is gone."
  reviewer: "orchestrator"
---

## Description

Update the GitHub Pages docs deployment workflow to use current GitHub Action versions that declare/run on Node 24, removing Node.js 20 deprecation warnings from successful deployments.

Context:
- The docs app itself already uses Node 24 via `site/.node-version` and `site/package.json` engines.
- The warning comes from older workflow action refs in `.github/workflows/docs.yml`: `actions/checkout@v4`, `actions/setup-node@v4`, `actions/upload-pages-artifact@v3` (internally `actions/upload-artifact@v4`), and `actions/deploy-pages@v4`.
- Latest checked Node 24-compatible majors were `actions/checkout@v7`, `actions/setup-node@v6`, `actions/upload-pages-artifact@v5`, and `actions/deploy-pages@v5`.

Expected work:
- Update `.github/workflows/docs.yml` action refs to current Node 24-compatible versions.
- Validate the workflow YAML and local docs build where practical.
- After merge/push, rerun or observe the docs deployment to confirm the Node 20 warning is gone.
