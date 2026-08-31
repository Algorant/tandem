---
id: task-100
type: task
title: "Fix Epic Board toggle ergonomics and filtering scope"
priority: "medium"
relatedFiles: ["tandem/src/tui.rs"]
tags: ["tui", "epic-board", "keyboard", "ux"]
createdAt: "2026-07-04T23:36:48Z"
updatedAt: "2026-07-05T01:50:08Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-04T23:37:21Z"
  deliveredAt: "2026-07-05T01:08:10Z"
  deliverables: ["Commit 25306a7099e552fe1afa100594a00c9dabf4a91e: Fix Epic Board filtering and toggle.", "Updated `tandem/src/tui.rs` key handling, mode line, footer hints, help popup, and status text to use `b` for State/Epic Board arrangement switching instead of uppercase `E`.", "Simplified Epic Board row roles to only Epic and Child.", "Changed Epic Board entry construction to include only board-visible epic task parents and board-visible task children linked to those epics by `parentId`.", "Added regression tests for decision exclusion, unparented Validation exclusion, non-epic child exclusion, epic child inclusion, and `b`/`e`/`E` shortcut behavior."]
  validation:
    commands: ["Parent inspected Shep/Tandem state and worker report.", "Parent inspected commit/diff for `tandem/src/tui.rs`.", "git diff --check HEAD~1..HEAD passed.", "cargo fmt --manifest-path tandem/Cargo.toml --check passed.", "cargo test --manifest-path tandem/Cargo.toml passed: 122 passed, 0 failed."]
  summary: "Accepted after human TUI validation. User validated the Epic Board cleanup and said it is great."
  evidence: ["Worker reported clean shared `main` worktree after commit.", "Parent confirmed git status: `main...origin/main [ahead 1]` with no dirty files.", "Parent confirmed HEAD: 25306a7 Fix Epic Board filtering and toggle.", "FFF grep confirmed only `b` is bound for board arrangement toggle and help/footer labels reference `b Epic Board` / `b State Board`."]
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "Algorant"
  updatedAt: "2026-07-05T01:49:43Z"
completedAt: "2026-07-05T01:50:08Z"
completion:
  summary: "Fixed Epic Board filtering and toggle ergonomics in commit 25306a7. Epic Board now shows only active epic task parents and active task children linked to those epics, excludes decisions and unrelated/unparented tasks, and uses `b` instead of uppercase `E` for State/Epic Board arrangement switching."
  validation: "Automated validation: cargo fmt --manifest-path tandem/Cargo.toml --check; cargo test --manifest-path tandem/Cargo.toml (122 passed); git diff --check HEAD~1..HEAD. Human validation: user verified the TUI change and said it is validated and great."
  reviewer: "Algorant"
---

## Description

The Epic Board view needs a focused cleanup pass based on current TUI validation.

Observed issues:
- The uppercase `E` board-arrangement toggle is easy to confuse with lowercase `e` edit, and accidental keypresses are risky/confusing.
- The Epic Board currently shows unrelated rows such as decision documents and unparented Validation tasks.
- Epic Board should show epic parent tasks and their child tasks, not every unparented board-visible document.

Scope:
- Rethink the Epic Board entry/toggle key and footer/help labeling so it is harder to confuse with edit.
- Restrict Epic Board rows to actual epic groups: epic parent tasks plus tasks linked to those epics through `parentId`.
- Exclude decisions from Epic Board entirely.
- Exclude unrelated/unparented non-epic tasks from Epic Board unless a deliberate placeholder/empty-state behavior is chosen.

Acceptance criteria:
- Epic Board no longer lists `type: decision` documents.
- Epic Board no longer lists unrelated/unparented Validation tasks.
- The visible rows are limited to epic parents and their child tasks.
- The board-arrangement control is renamed/rebound or otherwise made clearly distinct from `e` edit in the UI footer/help.
- Regression tests cover decisions, unparented tasks, and epic children in Epic Board row construction.
