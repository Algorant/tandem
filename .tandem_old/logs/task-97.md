---
id: task-97
type: task
title: "Reduce docs landing page to a minimal Home"
priority: "medium"
parentId: "task-59"
references: ["decision-1"]
relatedFiles: ["docs/index.md", "site/src/content/docs/index.md", "site/src"]
tags: ["docs", "site", "content"]
createdAt: "2026-07-04T23:25:43Z"
updatedAt: "2026-07-08T01:27:52Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-08T01:13:09Z"
  deliveredAt: "2026-07-08T01:16:09Z"
  deliverables: ["Commit b1c1f55 simplifies docs landing page in docs/index.md.", "Hero title changed to Tandem and tagline set to `Human and Agent Coordination`.", "Kept the existing SVG/logo mark.", "Removed hero CTA actions and stripped the overview cards, quick loop, role cards, and status body content.", "Left one short description sentence and preserved the no-duplicate-H1 convention."]
  validation:
    commands: ["Worker: git status --short clean after commit.", "Parent: git diff --check HEAD~1..HEAD passed.", "Parent: git merge-tree against current main detected no textual conflicts.", "Parent: cd site && bun install --frozen-lockfile && bun run build && bun run check:links passed; built 15 pages and checked 593 internal docs links. Known non-blocking Starlight 404 warning appeared."]
  summary: "Accepted task-97 after human visual review: minimal docs Home looks validated."
  evidence: ["Branch/worktree: shep/task-97-reduce-docs-landing-page-to-a-minimal-ho @ /home/ivan/.pi/agent/worktrees/tandem/task-97-reduce-docs-landing-page-to-a-minimal-ho", "Commit: b1c1f55d52737d35a87daa460ffba1eef7d42302 Simplify docs landing page"]
  filesChanged: ["docs/index.md"]
  reviewer: "user"
  updatedAt: "2026-07-08T01:27:46Z"
completedAt: "2026-07-08T01:27:52Z"
completion:
  summary: "Reduced docs landing page to a minimal Home with sparse tagline, retained logo, no CTA buttons, and minimal body copy after human visual validation."
  filesChanged: ["docs/index.md"]
  validation: "Human visual review approved. Automated validation passed: git diff --check; cd site && bun install --frozen-lockfile && bun run build && bun run check:links; merge-tree against current main had no textual conflicts."
  reviewer: "user"
---

## Description

Make the default docs landing page intentionally sparse.

Scope:
- Use the tagline: "Human and Agent Coordination".
- Keep the logo and one short description.
- Remove hero CTA buttons, including Start the quickstart, Browse concepts, and duplicate View GitHub.
- Strip the overview body content; leave it empty/minimal unless a deliberate placeholder is needed.
- Do not replace removed copy with generic marketing or protocol jargon.
