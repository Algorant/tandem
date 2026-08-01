---
id: task-75
type: task
title: "Explore configurable TUI badge shapes"
priority: "medium"
references: ["task-73"]
relatedFiles: ["tandem/src/tui/theme.rs", "tandem/src/tui.rs", "docs/tui/index.md", "tandem/README.md", "tandem/examples/themes/default-dark.toml", "tandem/examples/themes/verdigris.toml"]
tags: ["tui", "theme", "config", "badges", "design"]
createdAt: "2026-06-30T19:55:44Z"
updatedAt: "2026-07-01T04:21:23Z"
accord:
  status: "failed"
  deliveredAt: "2026-07-01T03:10:03Z"
  deliverables: ["Added BadgeShape square/rounded support in tandem/src/tui/theme.rs with root badge_shape and [badges] shape config parsing.", "Rounded labels use ASCII parentheses and compose with existing badge styles; text style ignores shape.", "Updated README/spec/docs and example themes."]
  validation:
    commands: ["Parent ran cd tandem && cargo test: passed 99/99.", "Parent ran git diff --check: passed.", "Parent inspected implementation and docs diffs."]
  summary: "Verified configurable TUI badge shape implementation and docs. Code/tests pass, but visual/design judgment remains for human validation as required."
  evidence: ["git diff -- tandem/src/tui/theme.rs tandem/README.md tandem/plan/spec.md tandem/examples/themes/default-dark.toml tandem/examples/themes/verdigris.toml docs/tui/index.md", "cd tandem && cargo test"]
  filesChanged: ["tandem/src/tui/theme.rs", "tandem/README.md", "tandem/plan/spec.md", "tandem/examples/themes/default-dark.toml", "tandem/examples/themes/verdigris.toml", "docs/tui/index.md"]
  reason: "Human rejected both attempted rounded badge designs. Parentheses looked like bracket text; glyph-cap approach also looked poor and did not blend with priority/status badge colors. Scope is being abandoned rather than reworked further."
  updatedAt: "2026-07-01T04:21:19Z"
completedAt: "2026-07-01T04:21:23Z"
completion:
  summary: "Abandoned configurable rounded TUI badge shapes after failed visual validation. Parentheses were rejected as plain bracket text, and the glyph-cap approach was also rejected because it looked poor and did not blend with badge colors. No further rework desired."
  validation: "Human requested marking the task failed/equivalent and moving it out of active work rather than continuing rework."
  reviewer: "human"
---

## Description

Investigate and implement, if practical, configurable TUI badge shapes so badge/chip rendering is not limited to squared-off blocks. Desired direction: add a rounded-looking option for badge styles where terminal constraints allow it.

Context: task-73 adds configurable badge display styles, but current badge shapes are visually squared off. A rounded option may improve fit with softer/transparent themes. This may require design thought because terminal cells cannot render true rounded rectangles; options may include rounded bracket/glyph approximations, softer separators, ghost-chip outlines, or style-specific shape behavior.

Acceptance criteria:
- Evaluate how rounded badge shapes can be represented legibly in terminal/Ratatui text spans.
- Add a documented config option if the design is viable, such as `badge_shape = "square" | "rounded"` or equivalent.
- Ensure shape configuration composes sensibly with badge style modes from task-73.
- Preserve accessible/scannable labels and avoid glyph choices that break common terminal fonts.
- Update docs/examples/tests where practical.
- Because this is visual/design-affecting, deliver to validation for human review rather than auto-completing.
