---
id: task-170
type: task
title: "Make Board task IDs consistently discoverable"
priority: "medium"
parentId: "task-168"
relatedFiles: ["tandem/src"]
tags: ["tui", "ui", "hierarchy"]
createdAt: "2026-07-23T14:32:22Z"
updatedAt: "2026-07-23T16:03:58Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-07-23T14:55:07Z"
  deliveredAt: "2026-07-23T14:58:13Z"
  deliverables: ["tandem/src/tui.rs: shared compact Board ID-chip helper and State Board integration", "tandem/src/tui.rs: Epic Board integration for Epic, Task, and Subtask rows", "tandem/src/tui.rs: regression assertions for nested child IDs"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test state_board_rows_label_subtasks_and_align_child_titles", "cd tandem && cargo test (162 passed)", "git diff --check", "Live local just dev inspection in Herdr workspace tandem, tab 2: State and Epic Board arrangements"]
  summary: "User visually validated the compact Board task-ID treatment."
  evidence: ["State Board showed #117, #133, #135… and nested #170. Epic Board showed #133 / #135 and #168 / #170 alongside relationship metadata."]
  reviewer: "user"
  updatedAt: "2026-07-23T16:03:49Z"
completedAt: "2026-07-23T16:03:58Z"
completion:
  summary: "Board task IDs now appear as compact #<number> chips across State and Epic Board rows; user validated visually."
  filesChanged: ["tandem/src/tui.rs"]
  validation: "cargo fmt --check; cargo test (162 passed); live Herdr tab 2 review"
  reviewer: "user"
---

## Description

Restore useful task-number/ID visibility throughout Board hierarchy rendering rather than showing the selected task ID only at the top of the screen. Establish a concise visual treatment that remains readable for Epics, Tasks, and Subtasks.
