---
id: task-91
type: task
title: "Audit remaining npm references and document Bun exception policy"
priority: "low"
references: ["decision-2", "task-66"]
relatedFiles: ["docs/guides/docs-site.md", "site/README.md", "justfile", ".github/workflows/docs.yml", "tandem/RELEASE.md"]
tags: ["docs", "bun", "audit", "maintenance"]
createdAt: "2026-07-04T13:04:16Z"
updatedAt: "2026-07-04T14:21:59Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-04T14:17:32Z"
  deliveredAt: "2026-07-04T14:21:32Z"
  deliverables: ["Confirmed no package-lock files remain outside ignored/generated areas.", "Confirmed remaining npm mentions in searched docs/workflow surfaces are intentional Bun exception-policy documentation only.", "Updated docs/guides/docs-site.md to require npm fallbacks be documented with attempted Bun avenues, failure reason, and revisit condition.", "Updated site/README.md with the same concise Bun exception-policy guidance."]
  validation:
    commands: ["Parent reran `fd -HI 'package-lock' . -E target -E site/node_modules -E .git`: no results.", "Parent reran `rg -n --hidden -S \"npm|package-lock|npm ci|npm install|npm run\" -g '!target/**' -g '!site/node_modules/**' -g '!.git/**' -g '!site/bun.lock' .`: only intentional exception-policy notes remain in docs/guides/docs-site.md and site/README.md.", "Parent reran `just site-build`: passed, 11 pages built.", "Parent reran `git diff --check`: passed."]
  summary: "PASS: Objective documentation/audit task verified. Remaining npm references are intentional exception-policy notes; package-lock files are gone; docs build and diff checks passed."
  evidence: ["Diff is limited to docs/guides/docs-site.md and site/README.md for task-91.", "Worker used shared working tree; no branch/worktree/commit created."]
  filesChanged: ["docs/guides/docs-site.md", "site/README.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-07-04T14:21:45Z"
completedAt: "2026-07-04T14:21:59Z"
completion:
  summary: "Audited remaining npm references and documented Bun exception policy after verification."
  filesChanged: ["docs/guides/docs-site.md", "site/README.md"]
  validation: "Verified `fd -HI 'package-lock' . -E target -E site/node_modules -E .git` found no files; `rg` npm/package-lock search only found intentional exception-policy notes; `just site-build` passed; `git diff --check` passed."
  reviewer: "orchestrator"
---

## Description

Audit repository docs, release instructions, and scripts for npm-specific guidance after the Bun migration.

Expected work:
- Search for `npm`, `package-lock`, and `npm ci/install/run` references across docs, site, justfile, release docs, and workflows.
- Replace stale npm guidance with Bun where validated.
- Preserve npm references only when they are historical, external examples, or documented exceptions with evidence per decision-2.
- Add a concise Bun exception-policy note to the relevant docs/site workflow page if not already covered.
