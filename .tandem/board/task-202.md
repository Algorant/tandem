---
id: task-202
type: task
title: "Overhaul TUI page"
state: "validation"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/tui/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T01:53:01Z"
updatedAt: "2026-08-05T02:45:36Z"
accord:
  status: "delivered"
  assignee: "worker-task-202-813deff9"
  claimedAt: "2026-08-05T02:26:10Z"
  deliveredAt: "2026-08-05T02:45:36Z"
  deliverables: ["docs/tui/index.md"]
  validation:
    commands: ["git diff --check passed", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Merged the TUI reference with screen placeholders, navigation, hierarchy behavior, validation, and complete theme/badge configuration."
  filesChanged: ["docs/tui/index.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:45:36Z"
assignee: "worker-task-202-813deff9"
---
## Approved TUI page direction

- Keep the technical TUI reference, with a clear `tandem tui` orientation at the top.
- Add a screen gallery near the top with placeholders for renders, GIFs, or images of every major screen and what it does:
  - Board / State Board
  - Epic Board
  - Logs
  - Rules
  - Decisions
  - Help and interaction states where useful
- Each screen placeholder should include a short caption describing the screen's purpose and key capabilities.
- Keep keyboard, mouse, hierarchy, filtering, validation, themes, and badge behavior documented below the gallery.
- Add a basic configuration section for themes and badges that documents every supported option:
  - Theme locations and selection: built-in themes, user TOML themes, user config, workspace overrides.
  - `theme`.
  - `transparent_background`.
  - `badge_style` and `[badges] style` compatibility.
  - `[board.badges] disabled`.
  - `[board.badges.tags.<tag>] label` and `tone`.
  - Supported tones: `accent`, `success`, `warning`, `error`, and `muted`.
  - Badge defaults and legacy configuration compatibility.
- Keep examples copyable and explain what each option changes.

Implementation remains pending until the page review is complete.