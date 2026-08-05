---
id: task-202
type: task
title: "Overhaul TUI page"
state: "in-progress"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/tui/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T01:53:01Z"
updatedAt: "2026-08-05T02:26:12Z"
accord:
  status: "claimed"
  assignee: "worker-task-202-813deff9"
  claimedAt: "2026-08-05T02:26:10Z"
  updatedAt: "2026-08-05T02:26:10Z"
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