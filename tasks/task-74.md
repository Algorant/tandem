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
  updatedAt: "2026-10-12T16:29:18Z"
createdAt: "2026-10-12T15:51:24Z"
updatedAt: "2026-10-12T16:29:18Z"
---

## Description

Approved task-70 follow-up. Add a shared app-layer local BoardSummary query and an all-workspaces TUI view, one row per configured board: todo/in-progress/validation, blocked or attention, last local change, sync status and per-entry errors. Define counts consistently with existing protocol/app meanings (avoid conflating delivered with validation). Sort by attention then local activity; Enter opens the chosen board through the switcher. Local reads only, no network fetch; refresh slowly or on demand, not on the active-board 250 ms loop. Do not add a second protocol parser or silently omit broken entries. Kickoff approved by Algorant via Herdra: implement in order 71 → 75. Visual Tasks remain delivered for Algorant's review, with actual screenshots in Sideshow. Keep docs overhaul task-76 paused.
