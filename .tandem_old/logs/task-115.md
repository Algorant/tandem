---
id: task-115
type: task
title: "Fix Epic Board child task ID truncation"
priority: "medium"
references: ["task-100"]
relatedFiles: ["tandem/src/tui.rs"]
tags: ["tui", "bug", "epic", "ui"]
createdAt: "2026-07-08T01:30:45Z"
updatedAt: "2026-07-08T01:51:05Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-08T01:31:14Z"
  deliveredAt: "2026-07-08T01:36:49Z"
  deliverables: ["Commit 4191d90 fixes Epic Board child metadata clipping in tandem/src/tui.rs.", "Adjusted Epic Board content width to reserve the List highlight symbol width.", "Updated board row width accounting so right-side metadata (child task ID) is preserved and title text yields/truncates first.", "Added regression test `board_row_preserves_right_metadata_when_title_space_is_tight` checking `task-102` remains fully visible under tight width."]
  validation:
    commands: ["Worker: cargo fmt --manifest-path tandem/Cargo.toml --check passed.", "Worker: targeted regression and Epic Board tests passed.", "Worker: cargo test --manifest-path tandem/Cargo.toml passed: 123 tests.", "Parent: git diff --check HEAD~1..HEAD passed.", "Parent: cargo fmt --manifest-path tandem/Cargo.toml --check passed.", "Parent: cargo test --manifest-path tandem/Cargo.toml passed: 123 tests.", "Parent: git merge-tree against current main detected no textual conflicts."]
  summary: "Accepted task-115 after human visual TUI review: Epic Board child task IDs now render fully."
  evidence: ["Branch/worktree: shep/task-115-fix-epic-board-child-task-id-truncation @ /home/ivan/.pi/agent/worktrees/tandem/task-115-fix-epic-board-child-task-id-truncation", "Commit: 4191d90154d4d986cddcd86d64550429dadb93ce Fix epic board child metadata clipping"]
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "user"
  updatedAt: "2026-07-08T01:50:45Z"
completedAt: "2026-07-08T01:51:05Z"
completion:
  summary: "Fixed Epic Board child task ID truncation by preserving right-side metadata width and accounting for the list highlight symbol."
  filesChanged: ["tandem/src/tui.rs"]
  validation: "Human visual TUI review approved. Automated validation passed: cargo fmt --manifest-path tandem/Cargo.toml --check; cargo test --manifest-path tandem/Cargo.toml; git diff --check; merge-tree against current main had no textual conflicts."
  reviewer: "user"
---

## Description

Bug: In the TUI Board Epic arrangement, nested child task rows truncate the last character/digit of the task ID in the far-right metadata column. Screenshot evidence showed child IDs under task-101 rendered as `task-10` instead of likely `task-102`/`task-103`/etc., and selected task-98 rendered as `task-9`.

Scope:
- Inspect Epic Board row rendering/layout width calculations in `tandem/src/tui.rs`.
- Ensure child task IDs render fully in the right-side metadata column without clipping the last character.
- Preserve existing Epic Board layout, selection behavior, and parent row summary rendering.
- Add or update focused tests if feasible for row layout/truncation behavior.
- Validate with `cargo fmt --manifest-path tandem/Cargo.toml --check`, relevant cargo tests, and manual/visual TUI review if needed.
