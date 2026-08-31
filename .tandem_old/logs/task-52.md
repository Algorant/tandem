---
id: task-52
type: task
title: "Add Board filtering by tag and severity"
priority: "medium"
relatedFiles: ["tandem/src/tui.rs", "tandem/plan/spec.md"]
tags: ["tui", "board", "filtering", "ux"]
createdAt: "2026-06-28T22:41:16Z"
updatedAt: "2026-06-29T02:11:09Z"
subtasks:
  - id: task-52-1
    title: "Define Board filter model for tag and priority/severity using existing task fields"
    completed: false
  - id: task-52-2
    title: "Add keyboard/UI flow to set, clear, and display active filters"
    completed: false
  - id: task-52-3
    title: "Filter Board rows without breaking state subview counts or selection behavior"
    completed: false
  - id: task-52-4
    title: "Add tests for tag and priority filter behavior"
    completed: false
  - id: task-52-5
    title: "Update TUI docs/spec for the filter controls"
    completed: false
  - id: task-52-6
    title: "Run cargo fmt, cargo test, and cargo build"
    completed: false
accord:
  status: "accepted"
  deliveredAt: "2026-06-29T01:49:41Z"
  deliverables: ["Prominent Board active-filter bar", "Focused Board filter render test"]
  validation:
    commands: ["cd tandem && cargo fmt --check && cargo test"]
  summary: "Accepted after visual review; Board filter visibility rework is slick and good."
  evidence: ["/home/ivan/dev/projects/tandem-worktrees/rework-board-filter-52 commit 983c7d4", "cd tandem && cargo fmt --check && cargo test (71 passed)"]
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "human"
  updatedAt: "2026-06-29T02:11:04Z"
completedAt: "2026-06-29T02:11:09Z"
completion:
  summary: "Accepted after visual review. Board filter visibility is now clear and polished; active filters are prominent without being jarring."
  filesChanged: ["tandem/src/tui.rs"]
  validation: "Human visual review accepted; merged cargo test suite passed."
---

## Description

Add TUI Board filtering so users can narrow visible active tasks by tag and by severity/priority. The Board should make the active filter visible, keep keyboard-first ergonomics, and avoid hiding context in a surprising way. Severity should map to existing priority values unless a separate severity field is intentionally introduced later.

## Feedback

### 2026-06-29 — Rework requested

Human visual review accepts the direction and functionality, but requests a visual polish pass.

- The tag/priority filtering behavior is useful and the implementation direction is great.
- Active search/filter criteria need more visual appeal and importance.
- Do not bury active filter state in the footer next to hotkeys.
- Explore clearer presentation such as:
  - a small popup or transient filter panel,
  - a prominent filter bar at the top of the Board/list box,
  - or another visually distinct but minimalist treatment.
- When a search criterion or filter is applied, it should be immediately noticeable without being jarring.
