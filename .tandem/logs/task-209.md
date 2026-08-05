---
id: task-209
type: task
title: "Add and verify bug, feat, and chore Board badges"
priority: "medium"
relatedFiles: [".tandem/config.toml", "tandem/src/tui/theme.rs", "tandem/src/tui/board/mod.rs", "docs/tui/index.md", "tandem/README.md"]
tags: ["ui", "theme", "taxonomy"]
createdAt: "2026-08-05T15:04:12Z"
updatedAt: "2026-08-05T16:26:58Z"
accord:
  status: "accepted"
  assignee: "worker-task-209-b07b1f1f"
  claimedAt: "2026-08-05T16:18:29Z"
  deliveredAt: "2026-08-05T16:26:49Z"
  deliverables: ["Built-in BUG, FEAT, and CHORE Board badges with duplicate filtering, suppression, and override behavior.", "Theme-owned orange, sand/beige, and purple tones for default-dark and Verdigris.", "Documentation and checked-in theme examples for built-in badges and named tone configuration."]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml -- --check passed.", "cargo test --manifest-path tandem/Cargo.toml passed: 227 unit and 11 integration tests.", "cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings passed.", "just site-build passed: 18 pages built.", "Manual Board preview inspected in workspace wJ tab 2; BUG, FEAT, and CHORE rendered in the safe task-209 fixture."]
  summary: "Added BUG, FEAT, and CHORE as minimal built-in Board badges with theme-owned orange, sand, and purple tones and configurable overrides."
  evidence: ["Integrated commit b4f6b73 into main via Worktrunk.", "Reviewed implementation diff, palette anchors, built-in/config boundary, tests, documentation, and theme examples.", "Target working tree remained clean after validation."]
  filesChanged: [".tandem/config.toml", "docs/tui/index.md", "tandem/README.md", "tandem/examples/themes/default-dark.toml", "tandem/examples/themes/verdigris.toml", "tandem/src/tui/board/mod.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/theme.rs"]
  reviewer: "orchestrator"
  note: "Reviewed the integrated implementation and Board preview against the user's approved direction. BUG uses rework/burnt-copper orange, FEAT uses warm sand, and CHORE uses delivered/heather purple. Automated checks and docs build pass."
  updatedAt: "2026-08-05T16:26:52Z"
assignee: "worker-task-209-b07b1f1f"
completedAt: "2026-08-05T16:26:58Z"
completion:
  summary: "Added and verified built-in BUG, FEAT, and CHORE Board badges with theme-owned orange, sand, and purple palettes, configurable overrides, tests, examples, and documentation."
  filesChanged: [".tandem/config.toml", "docs/tui/index.md", "tandem/README.md", "tandem/examples/themes/default-dark.toml", "tandem/examples/themes/verdigris.toml", "tandem/src/tui/board/mod.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/theme.rs"]
  validation: "Reviewed integrated diff and Board preview; formatting passed; 227 unit and 11 integration tests passed; strict Clippy passed; site build produced 18 pages."
  reviewer: "orchestrator"
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
