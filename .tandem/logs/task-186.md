---
id: task-186
type: task
title: "Expose workflow-state badge styling through Tandem themes"
priority: "medium"
relatedFiles: ["tandem/src/tui.rs", "tandem/README.md", "tandem/plan/spec.md"]
tags: ["tui", "theme", "ui"]
createdAt: "2026-07-24T15:25:42Z"
updatedAt: "2026-07-24T17:38:01Z"
accord:
  status: "accepted"
  assignee: "worker-task-186-2eca5c00"
  claimedAt: "2026-07-24T16:08:58Z"
  deliveredAt: "2026-07-24T17:11:43Z"
  deliverables: ["tandem/src/tui/theme.rs", "tandem/src/tui.rs", "tandem/examples/themes/verdigris.toml", "tandem/README.md", "tandem/plan/spec.md"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test (167 passed)", "Live TUI restarted in Herdr tandem workspace tab 2 with the user Verdigris theme overrides loaded; WIP row selected for visual review."]
  summary: "User visually validated Verdigris WIP burnt-copper and VAL heather-purple state chips in the live Tandem TUI."
  filesChanged: ["tandem/src/tui/theme.rs", "tandem/src/tui.rs", "tandem/examples/themes/verdigris.toml", "tandem/README.md", "tandem/plan/spec.md"]
  reviewer: "ivan"
  updatedAt: "2026-07-24T17:37:50Z"
assignee: "worker-task-186-2eca5c00"
completedAt: "2026-07-24T17:38:01Z"
completion:
  summary: "Added same-file theme color aliases and configurable per-workflow-state Board chips; user visually validated Verdigris WIP and VAL colors in live TUI."
  filesChanged: ["tandem/src/tui/theme.rs", "tandem/src/tui.rs", "tandem/examples/themes/verdigris.toml", "tandem/README.md", "tandem/plan/spec.md"]
  validation: "cargo fmt --check; cargo test (167 passed); user visual validation in Herdr Tandem tab 2."
  reviewer: "ivan"
---
## Description

## Goal

Make the Board workflow-state badges/chips—currently rendered as `TODO`, `WIP`, and `VAL`—visually configurable through Tandem theme configuration while retaining the current labels and default appearance.

## Scope

- Expose per-state badge styling in the existing theme configuration surface, with distinct color overrides for every configured workflow state.
- Support color aliases declared earlier in the same theme file, so state badges and other theme settings can share a named palette value.
- Preserve current defaults when no override is configured.
- Cover configured workflow states without hard-coding only the default labels.
- Validate theme parsing, alias resolution, fallback behavior, and Board rendering.
- Document the supported configuration succinctly.

Do not rename the badge text or introduce workflow-state changes as part of this task.
