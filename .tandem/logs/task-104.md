---
id: task-104
type: task
title: "Add TUI subtask hierarchy view and context"
priority: "medium"
parentId: "task-101"
relatedFiles: ["tandem/src/tui.rs", "tandem/plan/spec.md"]
tags: ["tui", "subtasks", "relationships"]
createdAt: "2026-07-05T16:22:34Z"
updatedAt: "2026-07-14T00:53:07Z"
accord:
  status: "failed"
  assignee: "shep-task-104"
  claimedAt: "2026-07-12T13:48:01Z"
  deliveredAt: "2026-07-14T00:28:32Z"
  deliverables: ["Focused rebased commit 0edd67d65822812e608d126d638e1127db9d4a25 on shep/task-104-add-tui-subtask-hierarchy-view-and-conte", "Subtask Board arrangement and navigation/render integration", "Nested hierarchy, filtering, task-vs-non-task parent, and arrangement-cycle regression tests", "Updated tandem/plan/spec.md TUI behavior"]
  validation:
    commands: ["Parent inspected the full implementation and corrected root-selection logic", "cargo fmt --manifest-path tandem/Cargo.toml -- --check — passed", "cargo test --manifest-path tandem/Cargo.toml subtask_board_ — 3 passed", "cargo test --manifest-path tandem/Cargo.toml — 130 passed, 0 failed", "git diff HEAD^ HEAD --check — passed", "merge-tree against current main — no conflict markers", "Worker git status --short — clean; no unexpected files"]
  summary: "Implemented a third TUI Board arrangement for first-class task-to-task subtask hierarchies. Subtask Board cycles with `b`, renders arbitrary-depth active task trees, preserves matching ancestor context under filters, keeps Epic Board separate, and excludes generic decision/non-task parent relationships from subtask grouping."
  evidence: ["Branch: shep/task-104-add-tui-subtask-hierarchy-view-and-conte", "Worktree: /home/ivan/.pi/agent/worktrees/tandem/task-104-add-tui-subtask-hierarchy-view-and-conte", "Commit: 0edd67d65822812e608d126d638e1127db9d4a25", "Visual validation command: from /home/ivan/dev/projects/tandem run `/home/ivan/.pi/agent/worktrees/tandem/task-104-add-tui-subtask-hierarchy-view-and-conte/tandem/target/debug/tandem tui`, press `b` twice to enter Subtask Board, inspect nesting/filtering/selection/expansion/mouse behavior"]
  filesChanged: ["tandem/src/tui.rs", "tandem/plan/spec.md"]
  reason: "Human product review found the separate general Subtask Board and flat-ID assumptions do not match the intended epic/subtask UX or hierarchical designation model. The implementation commit is intentionally rejected and will not be merged."
  updatedAt: "2026-07-14T00:52:56Z"
completedAt: "2026-07-14T00:53:07Z"
completion:
  summary: "Closed as unimplemented after human product review rejected the separate Subtask Board direction. Commit 0edd67d was not merged; replacement work will follow the clarified parent-derived hierarchical subtask ID model and Epic Board-focused UX."
  validation: "Human review determined the implementation did not match intended product behavior. Automated tests passed, but visual/product validation rejected the direction; no task-104 code was integrated."
  reviewer: "Algorant"
---

## Description

Add or design a TUI subtask-oriented view over `parentId` relationships.

Scope:
- Provide a Subtask view/tree similar in spirit to Epic Board, but not limited to `kind: epic`.
- Show parent tasks and child/subtask tasks nested under them.
- Surface subtask context in validation/review/detail flows where useful.
- Keep Epic Board behavior distinct: epics remain `kind: epic`; subtasks are general parent-linked tasks.
- Include tests for nested relationships and display filtering.
