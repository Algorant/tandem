---
id: task-74
uid: 73438650-cd27-47f2-a40d-47f690c163b1
type: task
title: "Orchestrator mode: BoardSummary query and TUI overview"
state: todo
priority: "medium"
effort: "large"
blockers: ["task-73"]
references: ["task-70"]
relatedFiles: ["tandem/src/app/queries.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/state.rs"]
tags: ["tui", "orchestrator"]
accord:
  status: "ready"
  acceptance: ["A shared BoardSummary query returns local per-workspace counts, attention, last change, sync state and explicit per-entry errors with documented counting semantics.", "The TUI overview displays configured workspaces sorted by attention and supports opening a selected board.", "Overview refresh is bounded and local-only, without triggering remote sync or full-board loads every active-board tick.", "A missing or invalid workspace is represented as an error row rather than aborting the overview."]
  validation: ["$ just dev-check"]
  constraints: ["Frozen: revised ephemeral-workspace plumbing/functions require Algorant's approval before any further code, mockups, integration or push. Preserve current work; no discards/rollbacks."]
  updatedAt: "2026-10-12T17:18:06Z"
createdAt: "2026-10-12T15:51:24Z"
updatedAt: "2026-10-12T17:18:06Z"
---
## Frozen by Algorant via Herdra

Do not implement, merge, push, create implementation Workers, produce mockups, discard, clean up or roll back existing work until Algorant approves revised plumbing/functions. Preserve all commits, checkouts and current preview state. New requirement: no dedicated permanent workspaces.toml; enabled/viewable workspaces must be ad-hoc, ephemeral and easy to change during a session. Discuss session roster, on-demand native Herdr project discovery and a repo picker first; no UI/button design yet. w/W remain tentative subject to conflicts. This direction overrides the earlier permanent-list implementation scope below, which is retained as history, not renewed approval.

Task-71 already landed as 1bfc31c and is archived accepted; its code is preserved, not rolled back. Task-72's retained unmerged Worker is clean at 4f6d5e7. Main retains temporary justfile/scripts/dev.sh preview helper mirrors; do not restore them while the no-rollback hold applies.

## Previous scope (on hold)


## Description

Approved task-70 follow-up. Add a shared app-layer local BoardSummary query and an all-workspaces TUI view, one row per configured board: todo/in-progress/validation, blocked or attention, last local change, sync status and per-entry errors. Define counts consistently with existing protocol/app meanings (avoid conflating delivered with validation). Sort by attention then local activity; Enter opens the chosen board through the switcher. Local reads only, no network fetch; refresh slowly or on demand, not on the active-board 250 ms loop. Do not add a second protocol parser or silently omit broken entries. Kickoff approved by Algorant via Herdra: implement in order 71 → 75. Visual Tasks remain delivered for Algorant's review, with actual screenshots in Sideshow. Keep docs overhaul task-76 paused.
