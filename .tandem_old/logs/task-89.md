---
id: task-89
type: task
title: "Update docs automation and GitHub Pages workflow to use Bun"
priority: "medium"
references: ["decision-2", "task-66", "task-88"]
relatedFiles: [".github/workflows/docs.yml", "justfile", "site/package.json", "site/bun.lock", "docs/guides/docs-site.md"]
tags: ["docs", "bun", "ci", "github-actions"]
createdAt: "2026-07-04T13:03:54Z"
updatedAt: "2026-07-04T14:00:08Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-04T13:40:07Z"
  deliveredAt: "2026-07-04T13:59:54Z"
  deliverables: ["Updated .github/workflows/docs.yml to setup Bun via oven-sh/setup-bun@v2 and run `bun install --frozen-lockfile` plus `bun run build`, preserving Pages artifact/deploy steps.", "Updated justfile docs recipes to use Bun for dev/build and release validation docs-site commands to use Bun install/build/audit.", "Updated docs/guides/docs-site.md to describe GitHub Pages workflow and just shortcuts using Bun.", "Updated tandem/RELEASE.md release checklist docs-site commands from npm to Bun."]
  validation:
    commands: ["Parent reran `just site-build`: passed, 11 Astro pages built.", "Parent parsed .github/workflows/docs.yml with Python/PyYAML: passed.", "Parent ran `rg -n \"npm (ci|install|run|audit)|package-lock\" .github/workflows/docs.yml justfile docs/guides/docs-site.md site/README.md tandem/RELEASE.md`: no stale matches.", "Parent ran `git diff --check -- .github/workflows/docs.yml justfile docs/guides/docs-site.md tandem/RELEASE.md`: passed."]
  summary: "PASS: Objective non-visual docs automation migration verified. GitHub Pages workflow, just recipes, and release docs now use Bun; validations passed."
  evidence: ["Diff shows only automation/release docs changes for task-89: .github/workflows/docs.yml, justfile, docs/guides/docs-site.md, tandem/RELEASE.md.", "Worker reported no commit created due shared working tree."]
  filesChanged: [".github/workflows/docs.yml", "justfile", "docs/guides/docs-site.md", "tandem/RELEASE.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-07-04T14:00:00Z"
completedAt: "2026-07-04T14:00:08Z"
completion:
  summary: "Updated docs automation and GitHub Pages workflow to use Bun after verification."
  filesChanged: [".github/workflows/docs.yml", "justfile", "docs/guides/docs-site.md", "tandem/RELEASE.md"]
  validation: "Verified `just site-build` passed; workflow YAML parsed; no stale `npm ci/install/run/audit` or `package-lock` references remain in touched automation docs; `git diff --check` passed. Files changed: .github/workflows/docs.yml, justfile, docs/guides/docs-site.md, tandem/RELEASE.md."
  reviewer: "orchestrator"
---

## Description

Update docs-site automation to use Bun by default in line with decision-2.

Expected work:
- Update `.github/workflows/docs.yml` to install Bun and run Bun-based install/build commands for the docs site, preserving GitHub Pages artifact/deploy behavior.
- Update `justfile` docs recipes (`site`, `site-build`, and release validation path where docs build is invoked) to use Bun commands and Bun lockfile semantics.
- Keep Node version checks only where needed for Astro runtime compatibility; avoid blocking newer supported Node versions.
- Validate local `just site-build` or equivalent Bun build and document any GitHub Actions-specific assumptions.
