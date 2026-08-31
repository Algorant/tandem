---
id: task-118
type: task
title: "Add progress bars to Board Epic view"
priority: "medium"
references: ["task-100"]
relatedFiles: ["tandem/src/tui"]
tags: ["tui", "epic", "board", "progress"]
createdAt: "2026-07-10T12:41:01Z"
updatedAt: "2026-07-23T19:00:57Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-07-23T16:25:38Z"
  deliveredAt: "2026-07-23T19:00:48Z"
  deliverables: ["tandem/src/tui.rs selected-Epic Board header progress bar", "tandem/src/tui/theme.rs configurable colors.progress token"]
  validation:
    commands: ["cargo fmt --check", "cargo test (162 passed)", "live Herdr tab 2 visual verification", "v0.6.2 release validation suite"]
  summary: "User validated the selected-Epic descendant progress bar, 24-segment resolution, and configurable progress theme color."
  evidence: ["User visually validated the progress display and 24-segment resolution."]
  reviewer: "user"
  updatedAt: "2026-07-23T19:00:53Z"
completedAt: "2026-07-23T19:00:57Z"
completion:
  summary: "Selected-Epic Board progress bar shipped in v0.6.2: descendant completion ratio, 24 segments, and configurable [colors] progress fill."
  filesChanged: ["tandem/src/tui.rs", "tandem/src/tui/theme.rs"]
  validation: "cargo fmt --check; cargo test (162 passed); full v0.6.2 release validation; user visual acceptance"
  reviewer: "user"
---

## Description

Add a compact progress bar in the Board's Epic view that summarizes child-task progress as a simple completed-to-total ratio, with completed and outstanding counts clear to the user.

Use currently unused/dead space in the Epic view where practical. Treat the exact placement and presentation as a small UX/design question to resolve during implementation or visual validation rather than locking it in now.

Acceptance criteria:
- Epic view shows progress for the active/displayed epic based on completed versus outstanding child tasks.
- The ratio and bar handle zero-child and all-complete cases cleanly.
- Placement does not crowd existing task content or controls.
- Final placement receives visual/UX validation.
