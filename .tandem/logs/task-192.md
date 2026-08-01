---
id: task-192
type: task
title: "Redesign the Rules view for readable rule scanning"
priority: "medium"
references: ["decision-10"]
relatedFiles: ["tandem/src/tui/rules.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/theme.rs", "tandem/plan/spec.md", "docs/tui/index.md"]
tags: ["tui", "rules", "ui", "visual", "readability"]
createdAt: "2026-07-31T02:56:00Z"
updatedAt: "2026-07-31T23:45:42Z"
accord:
  status: "accepted"
  assignee: "worker-task-192-7516b1a9"
  claimedAt: "2026-07-31T23:37:11Z"
  deliveredAt: "2026-07-31T23:43:42Z"
  deliverables: ["Integrated commit 3457fc1 on main", "Category-colored bordered list pane and title", "Dynamic row-count-based list height", "Large-category two-thirds cap and scrolling", "Preview fills all remaining space", "Border-aware mouse hit regions and content width", "Short-terminal and preview-closed fallback preserved", "Preview route cleared"]
  validation:
    commands: ["Orchestrator visually inspected small two-rule and larger nine-rule categories in wJ tab 2", "Worker full suite: 222 unit and 11 integration tests passed", "Worker 14 focused tests, strict Clippy, PTY smoke, and diff check passed", "Independent dynamic layout tests passed before and after integration", "Independent border-aware mouse hit test passed before and after integration"]
  summary: "Implemented and integrated the final dynamic Rules pane layout. The category-colored bordered list pane fits its rows for small categories, grows and scrolls up to a rounded two-thirds cap for large categories, and gives all remaining height to the preview. With preview closed or insufficient height, the bordered list uses the full area. Final visual approval remains with the user."
  filesChanged: ["tandem/src/tui/rules.rs"]
  reviewer: "Algorant"
  note: "User approved the final dynamic Rules list and preview pane design after direct terminal validation."
  updatedAt: "2026-07-31T23:45:34Z"
assignee: "worker-task-192-7516b1a9"
completedAt: "2026-07-31T23:45:42Z"
completion:
  summary: "Redesigned the Rules TUI into a dense category-colored list with an Enter-toggled full-rule preview, then refined it through user-reviewed Sideshow and terminal iterations. The final dynamic layout gives small categories a compact bordered list pane, caps large categories near two-thirds with scrolling, and lets the preview fill all remaining space."
  filesChanged: ["tandem/src/tui/rules.rs", "tandem/src/tui/theme.rs", "tandem/src/tui/input.rs", "tandem/src/tui/mod.rs", "tandem/plan/spec.md", "docs/tui/index.md"]
  validation: "User approved the final terminal UI. Worker full suite passed 222 unit and 11 integration tests; focused layout, mouse, keyboard, narrow-terminal, strict Clippy, PTY smoke, and post-integration checks passed."
  reviewer: "Algorant"
---
## Selected direction

Implement **Direction B — Wrapped rule cards**, selected by the user from the Sideshow concepts.

Reference: `http://localhost:8228/session/nEEQCHHMXxM/s/ReQMFf0z_mQ`

## Design intent

- Keep the existing one-pane category and list navigation model.
- Render each rule as a visually bounded, wrapped card instead of one truncated run-on row.
- Separate the rule ID, full readable rule text, and source metadata.
- Give the selected card a clear but restrained active treatment using existing theme semantics.
- Preserve category tabs, counts, keyboard and mouse navigation, add/edit/delete prompts, reload selection, and source context.
- Adapt card height and metadata placement for narrow terminals rather than introducing a permanent second pane.
- Accept fewer simultaneously visible rules as the trade-off for substantially better readability.

## Validation requirement

This is visual TUI work. Automated tests must cover layout and interaction behavior, but the completed implementation must remain in validation for direct terminal review against the selected Sideshow direction.
