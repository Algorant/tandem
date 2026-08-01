---
id: task-141
type: task
title: "Align TUI hierarchy rendering with canonical roles and IDs"
priority: "high"
blockers: ["task-140"]
references: ["decision-7", "task-132"]
relatedFiles: ["tandem/src/tui.rs", "tandem/src/tui/review.rs", "tandem/src/tui/logs.rs", "tandem/plan/spec.md", "justfile"]
tags: ["rust", "tui", "hierarchy", "ux", "visual", "ids"]
createdAt: "2026-07-15T19:44:53Z"
updatedAt: "2026-07-22T03:56:52Z"
parentId: "task-134"
accord:
  status: "accepted"
  assignee: "shep-task-141"
  claimedAt: "2026-07-22T02:54:34Z"
  deliveredAt: "2026-07-22T03:51:03Z"
  deliverables: ["Focused commit 13f90909f7fcde2961bf9cd6022e53afa2ede552 on shep/task-141-align-tui-hierarchy-rendering-with-canon", "Canonical State Board, Epic Board, Logs, Review/Validation, relationship detail, filtering, logged-parent traversal, diagnostics, and narrow-width behavior", "Routed human preview at /tmp/tandem-task-141-preview via `just dev`"]
  validation:
    commands: ["cargo fmt --check — passed", "cargo check — passed", "cargo test mixed_case_task_and_epic_values_are_custom_or_invalid_not_canonical_roles -- --nocapture — 1 passed", "cargo test — 154 passed", "cargo clippy --all-targets — passed with existing 27 binary/28 test warning backlog", "git diff --check main...HEAD — passed", "git show --check HEAD — passed", "Worker preview strict list — passed with 8 documents"]
  summary: "Human visual validation approved through the routed `just dev` preview. Parent review and automated validation passed; commit 13f9090 was integrated on main as 3c1712e. State Board, Epic Board, relationship context, Logs, diagnostics, and narrow-width behavior are accepted."
  evidence: ["Parent reviewed the full task-141 diff and the final 0776836..13f9090 amendment", "Final amendment removed all TUI is_epic_task usage and selects Epic roots/badges/headings from cached TaskRole context", "Human visual/product judgment is intentionally pending"]
  filesChanged: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/review.rs", "tandem/plan/spec.md"]
  reviewer: "user-and-parent-orchestrator"
  updatedAt: "2026-07-22T03:56:45Z"
completedAt: "2026-07-22T03:56:52Z"
completion:
  summary: "Integrated canonical TUI hierarchy rendering on main as 3c1712e after parent diff review, 154 passing Rust tests, strict real-workspace reads, pi-tandem relationship/runtime smokes, and explicit user approval of the routed `just dev` visual preview."
  filesChanged: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/review.rs", "tandem/plan/spec.md"]
  validation: "Human `just dev` validation approved. cargo fmt --check, cargo check, cargo test (154 passed), non-strict cargo clippy --all-targets, strict real-workspace list, pi-tandem relationship smoke, pi-tandem runtime smoke, and Git diff checks passed."
  reviewer: "user-and-parent-orchestrator"
---

## Description

This is a direct Task of Epic task-134. Apply the canonical decision-7 classifier to every TUI hierarchy and relationship surface.

Acceptance criteria:
- Epic Board renders globally numbered direct Epic children as Tasks without `SUB`; only parent-derived third-tier children render as Subtasks where a label is useful.
- State Board, selected/detail context, Review/Validation, Logs, filters, rollups, and completed-parent traversal distinguish Epic Tasks, Subtasks, and generic parent relationships.
- Keep the approved quiet State Board presentation from task-132; do not reintroduce procedural IDs or redundant role chips there.
- Invalid nested Epics, children beneath Subtasks, and role/ID mismatches surface actionable diagnostics rather than being flattened or accepted.
- Remove fixtures and tests that treat direct Epic children with hierarchical IDs as valid.
- Add focused rendering, navigation, filtering, reload, logged-parent, narrow-width, and invalid-structure tests; require human visual validation through `just dev`.
