---
id: task-54
type: task
title: "Add optional transparent theme background support"
priority: "low"
createdAt: "2026-06-29T00:19:43Z"
updatedAt: "2026-06-29T17:54:47Z"
accord:
  status: "accepted"
  deliveredAt: "2026-06-29T03:10:28Z"
  summary: "Added opt-in transparent_background theme setting that omits app/panel background fills when enabled, preserves opaque defaults when omitted, and documents the option."
  evidence: ["cd tandem && cargo fmt --check && cargo test (77 passed)", "Commit 8d753ec on branch herd-tui-interaction-theme-42-54"]
  filesChanged: ["tandem/src/tui/theme.rs", "tandem/README.md", "tandem/plan/spec.md", "docs/tui/index.md"]
  reviewer: "tui"
  updatedAt: "2026-06-29T17:13:05Z"
review.decidedAt: "2026-06-29T17:13:05Z"
review.reviewer: "tui"
review.status: "accepted"
completedAt: "2026-06-29T17:54:47Z"
completion:
  summary: "Accepted after review. Transparent background theme support works for current workspace needs."
  filesChanged: ["tandem/src/tui/theme.rs", "tandem/README.md", "tandem/plan/spec.md", "docs/tui/index.md"]
  validation: "Human review accepted; merged cargo test suite passed."
---

## Description

Problem: Tandem TUI themes currently do not expose an explicit way to make terminal backgrounds transparent.\n\nDesired outcome: add an optional theme setting that allows themes to use transparent/default terminal backgrounds, while keeping opaque backgrounds as the default behavior.\n\nAcceptance criteria:\n- Theme configuration supports an optional transparent background setting.\n- The setting is off by default / omitted themes preserve current rendering.\n- When enabled, background rendering should use the terminal default background where appropriate instead of forcing theme background fills.\n- Existing built-in and user themes remain compatible.\n- Document the option in the theme configuration docs/spec.
