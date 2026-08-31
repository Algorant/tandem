---
id: task-129
type: task
title: "Augment Epic Board for hierarchical first-class subtasks"
priority: "high"
parentId: "task-101"
blockers: ["task-126"]
references: ["decision-4", "task-104"]
relatedFiles: ["tandem/src/tui.rs", "tandem/plan/spec.md"]
tags: ["tui", "subtasks", "epic", "relationships"]
createdAt: "2026-07-14T00:55:06Z"
updatedAt: "2026-07-14T03:26:44Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-14T02:31:54Z"
  deliveredAt: "2026-07-14T03:25:32Z"
  deliverables: ["Focused commit ba6b4d8f78a34e78661f8e55b2de90a0fe2bb49f on shep/task-129-augment-epic-board-for-hierarchical-firs", "Concise aligned Epic Board visual grammar", "Recursive hierarchy, logged intermediary, Review relationship, filtering, legacy-flat, and narrow-width behavior with tests"]
  validation:
    commands: ["Worker: cargo fmt --check passed", "Worker: cargo test passed, 137 tests", "Worker: cargo clippy --all-targets passed with 25 existing warnings", "Worker: cargo build passed", "Worker: git diff --check and clean status passed", "Human visual validation approved for current scope with minor nits deferred"]
  summary: "PASS. Parent reviewed final commit ba6b4d8f78a34e78661f8e55b2de90a0fe2bb49f, independently passed format/build/clippy/137 tests/diff checks, integrated it to main, and the user visually approved the concise SUB/state/arrow layout for the current scope with minor nits deferred."
  evidence: ["Visual fixture /tmp/tandem-task-129-visual", "Validation binary in task-129 worktree", "User explicitly approved merge, push, and cleanup"]
  filesChanged: ["tandem/src/tui.rs", "tandem/src/tui/review.rs", "tandem/plan/spec.md"]
  reviewer: "Algorant"
  updatedAt: "2026-07-14T03:26:34Z"
completedAt: "2026-07-14T03:26:44Z"
completion:
  summary: "Augmented Epic Board with recursive first-class task descendants, compact aligned SUB/state rows, parent-to-child arrows, logged descendant rollups, legacy flat-child compatibility, logged-intermediary traversal, and correct Board/Review relationship context."
  filesChanged: ["tandem/src/tui.rs", "tandem/src/tui/review.rs", "tandem/plan/spec.md"]
  validation: "PASS. Parent reviewed and integrated ba6b4d8f78a34e78661f8e55b2de90a0fe2bb49f; independently passed cargo fmt --check, cargo build, cargo clippy --all-targets, all 137 Rust tests, git diff --check, and clean status. Algorant visually approved the final TUI for current scope and requested merge, push, and cleanup."
  reviewer: "Algorant"
---

## Description

Replace the rejected task-104 direction with an Epic Board-focused TUI implementation after hierarchical allocation exists.

Acceptance criteria:
- Do not add a separate general Subtask Board arrangement unless separately approved.
- Keep State and Epic Board arrangements.
- Epic Board shows epic task parents and their first-class task descendants using `parentId`, with clear nesting and actual hierarchical IDs such as `task-103-1` and `task-103-1-1`.
- Each child row makes its subtask designation and immediate parent context understandable; do not rely only on indentation where filtering or deep nesting could be ambiguous.
- Existing flat-ID children remain visible under their parent.
- Completed children contribute clear rollups/context without implying they remain active rows.
- Parent/subtask context remains visible in expanded detail and Validation/Review inspection.
- Generic decision/non-task parent relationships are not mislabeled as subtasks.
- Add navigation, filtering, nesting, legacy-flat, completed-child, and rendering tests.
- Require human visual/UX validation before acceptance.
