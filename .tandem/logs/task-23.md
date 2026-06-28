---
id: task-23
type: task
title: "Polish Board row visual hierarchy and badges"
priority: "high"
relatedFiles: ["tandem/src/tui.rs", "tandem/src/tui/theme.rs", "tandem/examples/themes/default-dark.toml", "tandem/examples/themes/verdigris.toml"]
tags: ["tui", "board", "ux", "theme"]
createdAt: "2026-06-28T03:31:41Z"
updatedAt: "2026-06-28T17:12:04Z"
subtasks:
  - id: task-23-1
    title: "Tune priority palette semantics: low green, medium neutral/blue, high red"
    completed: true
  - id: task-23-2
    title: "Render tags as clean highlighted chips without parentheses/bracket noise"
    completed: true
  - id: task-23-3
    title: "Suppress or de-emphasize default [task] row type metadata"
    completed: true
  - id: task-23-4
    title: "Replace bare A: accord marker with a cohesive accord badge"
    completed: true
  - id: task-23-5
    title: "Soften selected-row highlight so semantic colors remain legible"
    completed: true
  - id: task-23-6
    title: "Render expanded-summary tags as a simple #tag list instead of a quoted array"
    completed: true
  - id: task-23-7
    title: "Polish expanded-summary key/value styling with minimal distinct colors"
    completed: true
  - id: task-23-8
    title: "Format expanded-summary sections with readable wrapping or separators"
    completed: true
  - id: task-23-9
    title: "Strip selected-row highlight down to left cursor plus subtle title-only color"
    completed: true
  - id: task-23-10
    title: "Standardize priority and accord chip width/padding across HIGH/MED/LOW rows"
    completed: true
accord:
  status: "accepted"
  assignee: "herd:task23-board-row-polish"
  claimedAt: "2026-06-28T04:41:29Z"
  deliveredAt: "2026-06-28T17:09:00Z"
  deliverables: ["code:tandem/src/tui.rs:Board row renderer visual hierarchy/badge updates", "code:tandem/src/tui/theme.rs:theme-driven priority, badge, and selection styles", "theme:tandem/examples/themes/default-dark.toml:updated semantic examples if needed", "theme:tandem/examples/themes/verdigris.toml:updated semantic examples if needed"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test", "cd tandem && cargo build"]
  constraints: ["Keep the first pass focused on Board look-and-feel; avoid broad cross-view rewrites unless sharing theme primitives is necessary.", "Preserve keyboard/mouse behavior while changing rendering.", "Keep default styling usable in narrow terminals and terminal no-color fallback."]
  summary: "Final narrow Board expanded-row spacing pass: added blank-line separation between Tags, Summary, Files, Checklist, and footer while preserving accepted formatting and behavior."
  evidence: ["Validation passed: cd tandem && cargo fmt --check && cargo test && cargo build; git diff --check."]
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "ivan"
  note: "Visual review accepted after final expanded-row section spacing polish."
  updatedAt: "2026-06-28T17:12:04Z"
completedAt: "2026-06-28T17:12:04Z"
completion:
  summary: "Accepted Board row visual polish: priority/tag/accord chips, subtle selected row, expanded Tags/Summary/Files/Checklist sections with spacing, and preserved Tab/Enter behavior."
  filesChanged: ["tandem/src/tui.rs", "tandem/src/tui/theme.rs"]
  validation: "Human visual inspection accepted; automated validation passed: cargo fmt --check, cargo test, cargo build, git diff --check."
  reviewer: "ivan"
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

## Visual review feedback

Human review says the overall Board direction is much better and priority colors are good, but the delivered polish needs rework before acceptance:

- Expanded `Enter` summary tags currently render like a quoted array; render tags as a simple `#tag` list, either space-separated or comma-separated.
- Expanded summary should use minimal key/value styling: key in one color, value in another text color, not a busy treatment.
- Summary/body should read like a paragraph; sections should wrap cleanly or be separated with simple newlines/separators.
- Files can be either a compact comma-separated one-liner or a small bulleted list under a Files header.
- `Tab` details behavior works well, and `Enter` expand/collapse behavior works well; preserve both.
- Current selected-row highlight is too heavy and makes the row ugly again. Strip it down to a left index/cursor cue plus only a subtle title text color change for selected/hovered task.
- Accord marker direction is acceptable for now, including no marker for todo and front placement for in-progress, but chip sizing/padding should be standardized so HIGH/MED/LOW rows align uniformly.
