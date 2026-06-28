---
id: task-23
type: task
title: "Polish Board row visual hierarchy and badges"
state: todo
priority: "high"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/theme.rs", "tandem-tui/examples/themes/default-dark.toml", "tandem-tui/examples/themes/verdigris.toml"]
tags: ["tui", "board", "ux", "theme"]
createdAt: "2026-06-28T03:31:41Z"
updatedAt: "2026-06-28T03:32:39Z"
subtasks:
  - id: task-23-1
    title: "Tune priority palette semantics: low green, medium neutral/blue, high red"
    completed: false
  - id: task-23-2
    title: "Render tags as clean highlighted chips without parentheses/bracket noise"
    completed: false
  - id: task-23-3
    title: "Suppress or de-emphasize default [task] row type metadata"
    completed: false
  - id: task-23-4
    title: "Replace bare A: accord marker with a cohesive accord badge"
    completed: false
  - id: task-23-5
    title: "Soften selected-row highlight so semantic colors remain legible"
    completed: false
accord:
  status: "ready"
  assignee: "pi"
  deliverables: ["code:tandem-tui/src/tui.rs:Board row renderer visual hierarchy/badge updates", "code:tandem-tui/src/tui/theme.rs:theme-driven priority, badge, and selection styles", "theme:tandem-tui/examples/themes/default-dark.toml:updated semantic examples if needed", "theme:tandem-tui/examples/themes/verdigris.toml:updated semantic examples if needed"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Keep the first pass focused on Board look-and-feel; avoid broad cross-view rewrites unless sharing theme primitives is necessary.", "Preserve keyboard/mouse behavior while changing rendering.", "Keep default styling usable in narrow terminals and terminal no-color fallback."]
  updatedAt: "2026-06-28T03:32:39Z"
---

## Description

Board view is currently too noisy compared with the Brainfile reference screenshot. Tighten the row visual language before expanding more TUI features.

User-observed issues to address:
- Priority/severity colors should be semantically clear: low = bright green, medium = neutral/blue, high = red.
- Tags should look like clean tags/pills without parentheses or awkward spacing, using color highlighting rather than bracket noise.
- Hide or greatly de-emphasize `[task]` when the row type is the default task and no mixed document type context requires it.
- Replace the bare `A:` accord marker with a cohesive badge/chip treatment that reads as accord state without feeling purely functional.
- Use a muted selected-row treatment that preserves priority, tag, and accord colors instead of washing the whole row into one tone.

Acceptance direction:
- Board rows scan closer to the Brainfile reference: compact, legible, and less visually noisy.
- Selected rows keep metadata colors readable.
- Styling remains theme-driven where practical; update default/Verdigris examples if theme semantics change.
- Keep behavior focused on the Board view first; avoid broad cross-view rewrites unless a shared theme primitive is necessary.
