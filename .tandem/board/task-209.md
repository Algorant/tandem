---
id: task-209
type: task
title: "Add and verify bug, feat, and chore Board badges"
state: "in-progress"
priority: "medium"
relatedFiles: [".tandem/config.toml", "tandem/src/tui/theme.rs", "tandem/src/tui/board/mod.rs", "docs/tui/index.md", "tandem/README.md"]
tags: ["ui", "theme", "taxonomy"]
createdAt: "2026-08-05T15:04:12Z"
updatedAt: "2026-08-05T16:18:29Z"
accord:
  status: "claimed"
  assignee: "worker-task-209-b07b1f1f"
  claimedAt: "2026-08-05T16:18:29Z"
  updatedAt: "2026-08-05T16:18:29Z"
assignee: "worker-task-209-b07b1f1f"
---

## Description

Audit the Board tag-badge path for common repository work tags and make `bug`, `feat`, and `chore` render consistently. Decide and document whether these belong in Tandem's minimal built-in defaults or in the repository's `.tandem/config.toml`, without weakening the existing project-tag opt-in model.

## Visual direction

- `bug`: orange. Anchor `default-dark` to the existing accord rework orange (`#fb923c`) and Verdigris to its burnt-copper direction (`#c96f3d`). Do not use warning yellow or error red.
- `feat`: warm beige or sand. Use Verdigris warning/ready (`#e6bf86`) as the main direction and choose a compatible warm neutral for `default-dark`.
- `chore`: purple. Anchor `default-dark` to delivered purple (`#c084fc`) and Verdigris to its validation heather direction (`#ad8294`).
- Keep these colors theme-owned. The references above define palette direction; do not hard-code RGB values in Board rendering.

## Acceptance criteria

- `bug`, `feat`, and `chore` each render as a Board badge when configured or selected as defaults.
- Add or reuse named theme palette roles that produce the requested orange, beige, and purple treatments in both `default-dark` and `verdigris`.
- Define compatible config behavior for these badge tones. Preserve existing `accent`, `success`, `warning`, `error`, and `muted` settings.
- Preserve badge-style behavior across muted, accent, text, ghost, solid, and no-color modes.
- Add focused tests for default labels and colors, configured overrides, disabled badge handling, and duplicate/built-in filtering as applicable.
- Update TUI badge documentation and checked-in examples for the chosen default/config boundary and tone vocabulary.
- Perform an automated test run and a manual Board visual smoke check.
