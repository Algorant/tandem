---
id: task-76
type: task
title: "Remove configurable TUI badge style support"
priority: "high"
relatedFiles: ["tandem/src/tui/theme.rs", "tandem/src/tui.rs", "tandem/README.md", "tandem/plan/spec.md", "docs/tui/index.md", "tandem/examples/themes/default-dark.toml", "tandem/examples/themes/verdigris.toml", "tandem/GITHUB_RELEASE_NOTES.md"]
tags: ["tui", "theme", "config", "badges", "cleanup"]
createdAt: "2026-07-01T15:33:37Z"
updatedAt: "2026-07-01T16:09:30Z"
accord:
  status: "accepted"
  deliveredAt: "2026-07-01T16:09:16Z"
  deliverables: ["Updated tandem/src/tui.rs board-row comment to say chips use fixed saturated filled rendering as visual scan signals.", "Configurable badge style implementation/docs removal remains in place from prior pass."]
  validation:
    commands: ["Parent reran cd tandem && cargo check: passed with no warnings shown.", "Parent reran cd tandem && cargo test: passed, 98/98.", "Parent reran git diff --check: passed.", "Parent grep for removed feature references found only intentional strict-removal test references to badge_style."]
  summary: "Accepted: strict configurable badge-style removal is complete. Implementation/docs references are removed except intentional test coverage for unknown-key behavior, stale comment is fixed, and validations pass."
  evidence: ["git diff -- tandem/src/tui.rs", "grep: muted fill|accent rail|text label|ghost chip|legacy solid|badge_style|[badges] style|BadgeStyle|parse_badge_style|mix_color"]
  filesChanged: ["tandem/src/tui.rs", "tandem/src/tui/theme.rs", "tandem/README.md", "docs/tui/index.md", "tandem/plan/spec.md", "tandem/examples/themes/default-dark.toml", "tandem/examples/themes/verdigris.toml", "tandem/GITHUB_RELEASE_NOTES.md"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-07-01T16:09:22Z"
completedAt: "2026-07-01T16:09:30Z"
completion:
  summary: "Removed configurable TUI badge style support. Badge rendering is fixed saturated filled style again; badge_style and [badges] style are no longer parsed specially and produce ordinary theme warnings. Removed docs/examples/release-note advertising and resolved the badge_style_mode dead-code warning."
  validation: "cargo check passed without the badge_style_mode warning; cargo test passed 98/98; git diff --check passed; grep found only intentional strict-removal test references."
  reviewer: "pi-orchestrator"
---

## Description

Remove the regretted configurable TUI badge style feature from the CLI/TUI implementation and documentation. Strict removal is desired: existing `badge_style = ...` or `[badges] style = ...` should become normal unknown-key/theme warnings rather than being silently accepted. Restore fixed legacy badge rendering and remove dead-code warnings such as `badge_style_mode` being unused. Scope includes source, tests, README/spec/docs/examples, and release-note references as appropriate. Validate with `cd tandem && cargo check`, `cd tandem && cargo test`, and relevant docs/git checks.
