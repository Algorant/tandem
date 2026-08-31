---
id: task-171
type: task
title: "Repair Board hierarchy expand/collapse rendering"
priority: "high"
parentId: "task-168"
relatedFiles: ["tandem/src"]
tags: ["tui", "ui", "hierarchy"]
createdAt: "2026-07-23T14:32:28Z"
updatedAt: "2026-07-23T16:04:05Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-07-23T15:10:00Z"
  deliveredAt: "2026-07-23T15:51:16Z"
  deliverables: ["tandem/src/tui.rs: same-state hierarchies remain collapsed by default; cross-state ancestor paths expand only when needed", "tandem/src/tui.rs: every State Board row renders a compact state chip and Board IDs remain visible", "tandem/src/tui.rs: removed obsolete cross-state render field"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test (162 passed)", "git diff --check", "Live local just dev verification in Herdr workspace tandem, tab 2"]
  summary: "User visually validated the hierarchy, state-chip, and active Subtask behavior."
  evidence: ["TODO: 28 tasks / 7 collapsed root rows, each with #ID and TODO. In Progress: #168 TODO context with #171 WIP clearly visible."]
  reviewer: "user"
  updatedAt: "2026-07-23T16:03:54Z"
completedAt: "2026-07-23T16:04:05Z"
completion:
  summary: "State Board hierarchy now collapses same-state roots by default, exposes cross-state active paths, and labels every row with its own state; user validated visually."
  filesChanged: ["tandem/src/tui.rs"]
  validation: "cargo fmt --check; cargo test (162 passed); live Herdr tab 2 review with #174 → #175 → #175-1 WIP fixture"
  reviewer: "user"
---

## Description

Repair the visual tree behavior for Epic → Task → Subtask rendering: expansion indicators, connector lines, indentation, and row alignment should clearly and consistently communicate hierarchy in expanded and collapsed states. Manually verify against the live Herdr TUI fixture.
