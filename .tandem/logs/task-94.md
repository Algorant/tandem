---
id: task-94
type: task
title: "Implement standalone vendorable Verdigris docs theme"
priority: "medium"
references: ["task-93", "decision-3", "task-68"]
relatedFiles: ["site/src/styles/gruvbox.css", "site/astro.config.mjs", "site/src/styles/shiki", "site/README.md", "docs/guides/docs-site.md"]
tags: ["docs", "theme", "verdigris", "site", "starlight", "implementation"]
createdAt: "2026-07-04T15:08:35Z"
updatedAt: "2026-07-04T16:55:51Z"
subtasks:
  - id: task-94-1
    title: "Wait for user approval of task-93 standalone Verdigris visual direction"
    completed: false
  - id: task-94-2
    title: "Decide local vendored CSS structure and naming"
    completed: false
  - id: task-94-3
    title: "Implement standalone Verdigris Starlight variables and scoped Markdown/content styling"
    completed: false
  - id: task-94-4
    title: "Include intentional Expressive Code/code block color treatment"
    completed: false
  - id: task-94-5
    title: "Update docs/site documentation for the new theme source and maintenance workflow"
    completed: false
  - id: task-94-6
    title: "Validate site build and diff hygiene"
    completed: false
accord:
  status: "accepted"
  assignee: "herd:task-94"
  claimedAt: "2026-07-04T15:39:09Z"
  deliveredAt: "2026-07-04T16:33:15Z"
  deliverables: ["Enhanced site/src/styles/verdigris.css with distinct heading roles (green H1, aqua H2, brass/ochre H3, moss H4, cream/detail H5, muted H6), calmer active sidebar state, richer restrained cards/callouts/tags/blockquotes/tables/lists, and stronger Expressive Code fallback/layout styling.", "Fixed code-block rendering path by setting Starlight Expressive Code emitExternalStylesheet: false, reordering Verdigris dark/light themes, adding CSS fallback for token color variables, clearing Astro .astro content cache during sync, and running Astro dev/build scripts with --force so stale rendered code fences do not reference removed hashed ec.*.css assets.", "Added canonical docs/guides/theme-tester.md under canonical docs/ and linked it from Starlight Guides navigation plus docs/guides/index.md. The page exercises H1-H6, body copy, internal/external links, inline code, fenced TS/sh/TOML code blocks, blockquotes, Starlight asides, Markdown-compatible cards/tags, lists, and tables.", "Updated docs/site maintenance documentation to describe the Verdigris theme tester, Expressive Code inline stylesheet choice, cache-force behavior, and vendorable theme boundaries.", "Preserved the standalone Verdigris vendoring direction and removed old Gruvbox workaround assets from active site wiring."]
  validation:
    commands: ["mise x node@24 -- just site-build — passed. Bun install used frozen lockfile with no changes; Astro build ran with --force, synced 11 Markdown files, and built 12 pages including /guides/theme-tester/. Build emitted the existing non-blocking 'Entry docs → 404 was not found.' message but exited successfully.", "git diff --check — passed with no whitespace errors.", "Source stale-reference check passed: no src/styles/gruvbox, gruvbox-light, gruvbox-dark, starlight-theme-gruvbox.LICENSE, ec.v4551, or ec.9oy1k references remain in source docs/site files outside ignored dist/cache/node_modules.", "Rendered code-block validation passed against site/dist: no external/missing ec.*.css stylesheet refs remain, old Night Owl/Gruvbox stale markers (#82AAFF, #B8BB26, ec.v4551, ec.9oy1k) are absent, Verdigris token colors are present, and the theme tester output includes inline Expressive Code styles, TS code block markup, Starlight aside markup, and theme card markup."]
  summary: "Accepted after user visual review. User said the reworked Verdigris docs theme is good enough to call done."
  evidence: ["Branch/worktree/commit status: same shared working tree, no commit created. This remains intentional for MVP/shared-tree delegation so the parent/orchestrator can review and integrate visually before committing.", "Caveat: git diff --stat omits untracked new files unless listed separately; new Verdigris files and the new theme tester page are explicitly included in filesChanged.", "Caveat: the docs sync path currently copies plain .md files only, so the theme tester uses Markdown-compatible raw HTML helpers from verdigris.css for cards/tags instead of MDX-only Starlight Card/Badge components.", "Caveat: Expressive Code external stylesheet emission is disabled to avoid stale hashed stylesheet references in the current Astro/Starlight content pipeline; code-block styles are now inlined per page.", "Visual/theme caveat: implementation validates/builds, but because this is product-facing theme work it should receive parent/user visual review before acceptance/completion."]
  filesChanged: ["docs/guides/docs-site.md", "docs/guides/index.md", "docs/guides/theme-tester.md", "site/README.md", "site/astro.config.mjs", "site/package.json", "site/scripts/sync-docs.mjs", "site/src/styles/gruvbox.css", "site/src/styles/shiki/gruvbox-dark-medium.jsonc", "site/src/styles/shiki/gruvbox-light-medium.jsonc", "site/src/styles/shiki/starlight-theme-gruvbox.LICENSE", "site/src/styles/README.md", "site/src/styles/verdigris.css", "site/src/styles/shiki/verdigris-dark.jsonc", "site/src/styles/shiki/verdigris-light.jsonc"]
  reviewer: "parent/user"
  updatedAt: "2026-07-04T16:55:43Z"
completedAt: "2026-07-04T16:55:51Z"
completion:
  summary: "Implemented and accepted standalone vendorable Verdigris docs theme. Replaced Gruvbox workaround with Verdigris CSS and Shiki themes, added theme tester page, fixed Expressive Code rendering, documented theme asset boundary, and validated docs build."
  validation: "User visually reviewed rework and said it is good enough to call done. Parent validation passed: git diff --check, just site-build built 12 pages, rendered theme tester contains Expressive Code, asides, cards, and Verdigris token color; stale removed Gruvbox/code stylesheet references absent from source."
  reviewer: "parent/user"
---

## Description

Follow-up implementation task after task-93 visual direction approval. Build a standalone Verdigris docs/site theme rather than continuing to heavily modify the Gruvbox workaround. Scope should include extracting/renaming theme CSS away from gruvbox-specific naming, defining the Verdigris palette and Starlight role mapping, including headings/links/tags/cards/callouts/sidebar/code blocks, preserving Expressive Code/Starlight compatibility, documenting vendoring/package boundaries, and keeping changes reviewable. Do not start until the task-93 Sideshow direction is approved by the user.
