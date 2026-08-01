---
id: task-66
type: task
title: "Add docs quality checks and contribution workflow"
priority: "low"
parentId: "task-59"
references: ["decision-1"]
relatedFiles: ["site/package.json", "site/README.md", "docs/guides/docs-site.md", ".github/workflows/docs.yml", "site/src/content/docs/.gitignore"]
tags: ["docs", "ci", "maintenance"]
createdAt: "2026-06-29T20:49:45Z"
updatedAt: "2026-07-04T18:55:50Z"
accord:
  status: "accepted"
  assignee: "herd:task-66"
  claimedAt: "2026-07-04T17:59:12Z"
  deliveredAt: "2026-07-04T18:40:53Z"
  deliverables: ["Dependency-free internal link checker at `site/scripts/check-links.mjs` validates built HTML local links, assets, and fragments while intentionally skipping external URLs for deterministic CI.", "Docs scripts in `site/package.json`: `check:links` and `check:docs`.", "GitHub Pages workflow now runs `bun run check:links` after `bun run build`.", "Contribution/docs quality workflow documented in `site/README.md` and `docs/guides/docs-site.md`, including local preview, sync behavior, quality gate, checklist, and ignored generated files.", "Canonical docs links updated from generated `.md` targets to built-site routes in `docs/index.md`, `docs/guides/index.md`, and task-66-owned link edits in `docs/guides/theme-tester.md`.", "Generated docs ignore allowlist corrected in `site/src/content/docs/.gitignore`; generated notice updated in `site/src/content/docs/README.txt`.", "Added `site/public/favicon.svg`."]
  validation:
    commands: ["`cd site && bun run check:docs` passed; built 12 pages and checked 460 internal docs links across 12 HTML files.", "`cd site && bun install --frozen-lockfile` passed with no changes.", "`node --check site/scripts/check-links.mjs` passed.", "`git diff --check` passed.", "`git ls-files -ci --exclude-standard site/src/content/docs` returned empty."]
  summary: "Accepted objective validated docs quality workflow work. Reviewed task-66 subset: internal link checker, docs quality scripts, CI link check step, docs contribution/checklist updates, generated docs ignore/notice fixes, and favicon asset."
  evidence: ["Branch/worktree/commit: shared `main` worktree; no separate branch; no commit.", "Risks/caveats: internal link checker intentionally skips external URLs to avoid transient remote/bot-blocking CI failures.", "Unrelated concurrent files NOT task-66: `AGENTS.md`, `README.md`, `docs/tui/index.md`, `plan/spec.md`, `protocol/plan/spec.md`, `tandem/README.md`, `tandem/RELEASE.md`, `tandem/plan/spec.md`, `tandem/src/main.rs`, `tandem/src/tui.rs`, `tandem/src/tui/decisions.rs`, `tandem/src/tui/theme.rs`.", "Concurrent same-file caveat: `docs/guides/theme-tester.md` includes task-66 link edits, but the TOML badge namespace example change (`[badges.tags.docs]` -> `[board.badges.tags.docs]`) is not mine/task-66."]
  filesChanged: [".github/workflows/docs.yml", "docs/index.md", "docs/guides/index.md", "docs/guides/theme-tester.md", "docs/guides/docs-site.md", "site/README.md", "site/package.json", "site/scripts/check-links.mjs", "site/public/favicon.svg", "site/src/content/docs/.gitignore", "site/src/content/docs/README.txt"]
  reviewer: "parent/orchestrator"
  updatedAt: "2026-07-04T18:55:09Z"
completedAt: "2026-07-04T18:55:50Z"
completion:
  summary: "Completed docs quality checks and contribution workflow. Added dependency-free built-site internal link checker, Bun docs quality scripts, CI link-check step, docs maintenance/checklist updates, generated content ignore/notice fixes, and favicon asset."
  validation: "Parent/orchestrator reviewed task-66 subset and reran `cd site && bun run check:docs`, `bun install --frozen-lockfile`, `node --check site/scripts/check-links.mjs`, `git diff --check` for task-66 files, and generated-docs ignored-file check; all passed/empty as expected."
  reviewer: "parent/orchestrator"
---

## Description

Make docs maintenance safe and easy: keep npm run build in CI, add/link-check or equivalent if practical, document local preview and sync behavior, add a docs update checklist, and ensure generated files stay ignored.
