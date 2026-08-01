---
id: task-96
type: task
title: "Simplify docs sidebar into a barebones information architecture"
priority: "medium"
parentId: "task-59"
references: ["decision-1"]
relatedFiles: ["site/astro.config.mjs", "site/src/content/docs", "docs/index.md"]
tags: ["docs", "site", "nav"]
createdAt: "2026-07-04T23:25:37Z"
updatedAt: "2026-07-08T00:48:20Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-08T00:10:03Z"
  deliveredAt: "2026-07-08T00:46:10Z"
  deliverables: ["Commit ffe3101 simplifies sidebar IA.", "Commit 862caea removes immediate top-level Markdown H1s after frontmatter so Starlight renders a single page heading.", "Merge commit 6caf273 brings in current main/task-112 release docs and resolves the AUR docs conflict while preserving both task-112 content and no-duplicate-H1 convention."]
  validation:
    commands: ["Worker: cd site && bun run build && bun run check:links passed after integration; built 15 pages and checked 617 internal docs links.", "Parent: git status --short --branch clean in worktree.", "Parent: git diff --check HEAD~1..HEAD passed.", "Parent: git merge-tree against current main reports no textual conflicts.", "Parent: script check found no docs Markdown page with an immediate '# ' heading after frontmatter."]
  summary: "Accepted task-96 after human visual review: docs sidebar IA and duplicate heading fix look good."
  evidence: ["Branch/worktree: shep/task-96-simplify-docs-sidebar-into-a-barebones-i @ /home/ivan/.pi/agent/worktrees/tandem/task-96-simplify-docs-sidebar-into-a-barebones-i", "Final branch HEAD: 6caf273 Merge branch 'main' into shep/task-96-simplify-docs-sidebar-into-a-barebones-i", "Preview command from worktree: cd /home/ivan/.pi/agent/worktrees/tandem/task-96-simplify-docs-sidebar-into-a-barebones-i && just docs"]
  filesChanged: ["site/astro.config.mjs", "docs/skills/index.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/extensions/index.md", "docs/guides/decisions.md", "docs/guides/docs-site.md", "docs/guides/index.md", "docs/guides/theme-tester.md", "docs/packaging/aur-tandem-bin.md", "docs/protocol/index.md", "docs/quick-start/index.md", "docs/reference/index.md", "docs/tui/index.md"]
  reviewer: "user"
  updatedAt: "2026-07-08T00:48:14Z"
completedAt: "2026-07-08T00:48:20Z"
completion:
  summary: "Simplified docs sidebar into a barebones IA and removed duplicate page headings after human visual review."
  filesChanged: ["site/astro.config.mjs", "docs/skills/index.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/extensions/index.md", "docs/guides/decisions.md", "docs/guides/docs-site.md", "docs/guides/index.md", "docs/guides/theme-tester.md", "docs/packaging/aur-tandem-bin.md", "docs/protocol/index.md", "docs/quick-start/index.md", "docs/reference/index.md", "docs/tui/index.md"]
  validation: "Human visual review approved. Automated validation passed: cd site && bun run build && bun run check:links; merge-tree against current main had no textual conflicts."
  reviewer: "user"
---

## Description

Replace the current overbuilt sidebar labels with a small skeleton.

Target shape:
- Home: default landing page.
- Quickstart: top-level main section/page.
- Overview: Spec, CLI, TUI, Concepts.
- Workflows: workflow-oriented guides only.
- Integrations: Extensions, Skills.

Acceptance notes:
- Keep the top bar/logo treatment unchanged.
- Remove redundant labels such as Start here, Core model, and Reference.
- Do not add filler page body copy while changing navigation.
