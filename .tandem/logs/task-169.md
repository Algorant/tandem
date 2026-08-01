---
id: task-169
type: task
title: "Show in-progress subtasks in the Board pane"
priority: "high"
parentId: "task-168"
relatedFiles: ["tandem/src"]
tags: ["tui", "ui", "hierarchy"]
createdAt: "2026-07-23T14:32:18Z"
updatedAt: "2026-07-23T14:54:39Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-07-23T14:33:23Z"
  deliveredAt: "2026-07-23T14:53:25Z"
  deliverables: ["tandem/src/tui.rs: State Board Subtask rows always render their compact workflow-state chip", "tandem/src/tui.rs: regression coverage asserts the in-progress Subtask row contains WIP"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test (162 passed)", "Live local just dev inspection in Herdr workspace tandem, tab 2: fixture row renders WIP"]
  summary: "User visually confirmed the WIP Subtask state marker is a beautiful solution."
  evidence: ["Herdr tab 2 In Progress pane showed: `└─  WIP   UI inspection fixture: in-progress subtask`."]
  reviewer: "user"
  updatedAt: "2026-07-23T14:54:35Z"
completedAt: "2026-07-23T14:54:39Z"
completion:
  summary: "State Board now projects in-progress Subtasks with auto-expanded ancestor context and explicit WIP state chips; visually accepted by the user."
  filesChanged: ["tandem/src/tui.rs"]
  validation: "cargo fmt --check; cargo test (162 passed); live Herdr tab 2 inspection"
  reviewer: "user"
---

## Description

Fix the mismatch where an in-progress Subtask is counted in the Board’s In Progress summary but has no visible Board component. Define and implement a clear representation that preserves hierarchy context and makes every counted active subtask discoverable in the appropriate pane.
