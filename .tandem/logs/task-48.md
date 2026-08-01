---
id: task-48
type: task
title: "Remove Board sub-pane position indicator text"
priority: "low"
createdAt: "2026-06-28T18:30:49Z"
updatedAt: "2026-06-29T01:37:32Z"
accord:
  status: "accepted"
  deliveredAt: "2026-06-29T00:19:14Z"
  summary: "Accepted after visual review; Board pane position indicator removal is good."
  evidence: ["Commit 1d95ef9 Tighten TUI footer hints", "Added assertion that Board footer no longer includes x/y progress text", "cd tandem && cargo fmt --check && cargo test (61 passed)"]
  filesChanged: ["tandem/src/tui.rs", "tandem/plan/spec.md"]
  reviewer: "human"
  updatedAt: "2026-06-29T01:37:19Z"
completedAt: "2026-06-29T01:37:32Z"
completion:
  summary: "Accepted after visual review. Board sub-pane x/y indicator text has been removed and focus remains clear through styling."
  filesChanged: ["tandem/src/tui.rs", "tandem/plan/spec.md"]
  validation: "Human visual review accepted; merged cargo test suite passed."
---

## Description

Problem: when switching Board columns/sub-panes such as todo, in-progress, and review, the UI shows selected pane text like 1/3, 2/3, 3/3. This appears to be debug/status text and is not useful.

Desired outcome: remove the x/y selected-pane indicator from the Board UI.

Acceptance criteria:
- No x/y pane indicator appears while navigating Board columns.
- Focused Board column remains clear through border/highlight/styling only.
- No user-facing Board information is lost.
