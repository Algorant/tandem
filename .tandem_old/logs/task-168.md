---
id: task-168
type: task
kind: "epic"
title: "Overhaul TUI Board hierarchy and visual clarity"
priority: "high"
relatedFiles: ["tandem/src"]
tags: ["tui", "ui", "hierarchy"]
createdAt: "2026-07-23T14:32:09Z"
updatedAt: "2026-07-23T16:06:22Z"
completedAt: "2026-07-23T16:06:22Z"
completion:
  summary: "Completed the TUI Board hierarchy/clarity overhaul: active Subtasks project with visible WIP state, Board task IDs are consistently discoverable, and hierarchy collapse/expansion behavior was visually validated."
  filesChanged: ["tandem/src/tui.rs"]
  validation: "All child tasks task-169, task-170, and task-171 completed after cargo fmt --check, cargo test (162 passed), and live Herdr tab 2 user validation."
  reviewer: "user"
---

## Description

Improve the Tandem TUI Board’s visual representation of Epic → Task → Subtask work. This Epic tracks reconciliation of state counters with visible rows, persistent task-ID discoverability, and clear/robust hierarchy expansion rendering. Verify changes against the live TUI fixture in Herdr workspace `tandem`, tab `2`.
