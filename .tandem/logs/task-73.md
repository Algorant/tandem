---
id: task-73
type: task
title: "Add configurable TUI badge display styles"
priority: "medium"
relatedFiles: ["tandem/src/tui/theme.rs", "tandem/src/tui.rs", "tandem/examples/themes/default-dark.toml", "tandem/examples/themes/verdigris.toml", "docs/tui/index.md", "tandem/README.md"]
tags: ["tui", "theme", "config", "badges", "design"]
createdAt: "2026-06-30T12:37:00Z"
updatedAt: "2026-06-30T20:06:49Z"
subtasks:
  - id: task-73-1
    title: "Define badge style config schema and defaults"
    completed: false
  - id: task-73-2
    title: "Implement render style variants for muted fill, accent rail, text-only, and ghost chip"
    completed: false
  - id: task-73-3
    title: "Apply style variants to Board priority and inline metadata badges"
    completed: false
  - id: task-73-4
    title: "Document configuration examples in TUI docs and example themes"
    completed: false
  - id: task-73-5
    title: "Add parsing/default tests or TUI smoke coverage"
    completed: false
accord:
  status: "accepted"
  assignee: "shep:task-73-badges"
  claimedAt: "2026-06-30T19:21:17Z"
  deliveredAt: "2026-06-30T19:59:00Z"
  deliverables: ["Renamed canonical default badge style value from `filled` to `muted` across docs, examples, and tests.", "Updated BadgeStyle enum/default references from Filled to Muted.", "Parser accepts canonical `muted` and keeps `filled`/`filled-muted` as compatibility aliases.", "Committed revision as 0502070 (Rename muted TUI badge style option)."]
  validation:
    commands: ["cargo test --manifest-path tandem/Cargo.toml passed: 98 tests", "Rework verified canonical badge_style values: muted, accent, text, ghost, solid", "Text style priority labels are padded/aligned consistently"]
  summary: "User visually accepted configurable TUI badge style work after rework."
  evidence: ["Visual/design work: requires human/orchestrator visual judgment before acceptance.", "No commit hash: worker did not commit.", "Working tree has interleaved changes from task-70/72/73."]
  filesChanged: ["docs/tui/index.md", "tandem/README.md", "tandem/plan/spec.md", "tandem/examples/themes/default-dark.toml", "tandem/examples/themes/verdigris.toml", "tandem/src/tui/theme.rs"]
  reviewer: "user"
  updatedAt: "2026-06-30T20:06:41Z"
review.decidedAt: "2026-06-30T20:06:22Z"
review.note: "The badge style names should be one word for clarity, rename to filled, accent, text, ghost, solid."
review.reviewer: "tui"
review.status: "accepted"
completedAt: "2026-06-30T20:06:49Z"
completion:
  summary: "Added configurable TUI badge styles with canonical values muted, accent, text, ghost, and solid; included compatibility aliases for previous names, docs/examples/spec updates, aligned text-style priority labels, and user-accepted visual rework."
  validation: "User accepted visual review; cargo test --manifest-path tandem/Cargo.toml passed with 98 tests; commits include 7f8fbb0, 3229d1b, and 0502070."
  reviewer: "user"
---

## Description

Add configuration options for softer TUI badge treatments so users can choose how priority/status/tag badges render in the Board and related TUI views.

Context: current solid, saturated badge backgrounds (for priority badges like HIGH/MED/LOW and inline tags like RESEARCH) are useful as cues but too visually dominant, especially with transparent/background-image terminal themes. Provide style modes based on the Sideshow mockups:

- `filled-muted`: keep the current filled badge shape but desaturate/lower contrast.
- `accent-rail`: remove broad fills and use a small colored rail/accent with subdued text.
- `text-only`: render priority/tag labels as colored text without a badge block.
- `ghost-chip`: keep a badge outline/chip shape with transparent fill and softer borders.

Acceptance criteria:
- Add a documented config key for badge rendering style, available from user config/theme/workspace theme loading where appropriate.
- Preserve a sensible default that is less bold than the current solid blocks, or preserve current behavior behind an explicit legacy/current option if needed.
- Apply the selected badge style consistently to priority badges and inline metadata/tag badges in the TUI Board list.
- Keep badge text scannable and accessible in dark/transparent terminal themes.
- Update built-in/example theme docs and the TUI docs with examples for each option.
- Include tests or smoke coverage for config parsing/defaulting and render-style selection where practical.

## Feedback

- 2026-06-30T19:50:53Z (tui): The badge style names should be one word for clarity, rename to filled, accent, text, ghost, solid.
