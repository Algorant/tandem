---
id: task-60
type: task
title: "Write polished landing, quickstart, and concepts docs"
priority: "low"
parentId: "task-59"
references: ["decision-1"]
relatedFiles: ["docs/index.md", "docs/concepts/index.md", "docs/guides/index.md", "docs/cli/index.md"]
tags: ["docs", "content"]
createdAt: "2026-06-29T20:49:19Z"
updatedAt: "2026-07-04T21:29:45Z"
accord:
  status: "accepted"
  assignee: "pi-docs-content"
  claimedAt: "2026-07-04T21:01:49Z"
  deliveredAt: "2026-07-04T21:26:47Z"
  deliverables: ["`docs/quick-start/index.md` added as a real top-level Quickstart covering install lanes, CLI/TUI-only first workflow, accord delivery/acceptance, completion/logs, and `tandem tui`.", "`docs/index.md` updated as landing page teaser/value prop linking to Quickstart instead of containing the full workflow.", "`docs/concepts/index.md` refocused around Board task states `todo`, `in-progress`, `validation`, accords, epics, rules, decisions, logs, workspace files, and daily loop.", "`docs/guides/index.md` pruned to current available guides and related starting points, without planned/maybe-later first-pass gaps.", "`docs/cli/index.md` pruned to first-pass CLI essentials and links to Quickstart.", "`site/astro.config.mjs` adds top-level Quickstart navigation near Overview."]
  validation:
    commands: ["Parent verified `git status --short --branch`: expected dirty shared main with `M docs/cli/index.md`, `M docs/concepts/index.md`, `M docs/guides/index.md`, `M docs/index.md`, `M site/astro.config.mjs`, and untracked `docs/quick-start/`.", "Parent reran `git diff --check`: passed with no output for tracked changes.", "Parent ran untracked quickstart whitespace check via `git diff --check --no-index /dev/null docs/quick-start/index.md`: no whitespace errors.", "Parent reran `cd site && bun run check:docs`: passed; built 13 pages including `/quick-start/index.html` and checked 565 internal links.", "Parent searched touched docs for out-of-scope planned items (`why-tandem`, daily workflow guide, split reference tree, MCP/API/library/schema/template/migration, etc.); no matches in touched docs.", "Worker reported CLI smoke in temp workspace passed for init/add/show/list/move/claim/deliver/validation/accept/complete/log/search; `tandem tui` not launched because interactive."]
  summary: "Accepted task-60 based on user validation of the delivered docs content. The approved implementation adds a real top-level Quickstart, updates the landing page, refocuses Concepts around TUI/Board workflow, prunes Guides and CLI to first-pass essentials, and adds Quickstart to site navigation."
  evidence: ["shep_check showed worker completed rework and reported changed files, validation, no blockers, and readiness for parent review.", "No separate branch/worktree/commit was used; changes are in the shared main working tree as expected for this delegated content implementation."]
  filesChanged: ["docs/quick-start/index.md", "docs/index.md", "docs/concepts/index.md", "docs/guides/index.md", "docs/cli/index.md", "site/astro.config.mjs"]
  reviewer: "Algorant"
  updatedAt: "2026-07-04T21:29:21Z"
completedAt: "2026-07-04T21:29:45Z"
completion:
  summary: "Completed docs content pass after user validation. Added top-level Quickstart with install lanes and CLI/TUI workflow, updated landing copy, refocused Concepts around Board states/accords/epics/rules/decisions/logs, pruned Guides and CLI pages to first-pass essentials, and added Quickstart to Starlight navigation."
  filesChanged: ["docs/quick-start/index.md", "docs/index.md", "docs/concepts/index.md", "docs/guides/index.md", "docs/cli/index.md", "site/astro.config.mjs"]
  validation: "User validation from Algorant. Parent verification passed: git diff --check, untracked quickstart whitespace check, `cd site && bun run check:docs` building 13 pages including /quick-start/ and checking 565 internal links; touched-doc search found no out-of-scope planned items."
  reviewer: "Algorant"
---

## Description

Turn the current terse overview into a user-facing introduction: what Tandem is, why local-first Markdown coordination matters, the mental model, and a 5-minute quickstart from install/init through adding, claiming, delivering, validating, and logging work.
