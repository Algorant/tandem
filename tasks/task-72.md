---
id: task-72
uid: 2906d301-e968-4665-bf36-68ae974376b0
type: task
title: "Orchestrator mode: TUI workspace switcher"
state: "in-progress"
priority: "medium"
effort: "medium"
blockers: ["task-71"]
references: ["task-70"]
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/state.rs", "tandem/src/app/background_sync.rs", "tandem/src/tui/pickers.rs", "tandem/src/project/workspaces.rs", "scripts/dev.sh", "justfile", "tandem/README.md", "tandem/src/tui/input.rs", "tandem/src/tui/bindings.rs"]
tags: ["tui", "orchestrator", "keyboard"]
accord:
  status: "claimed"
  acceptance: ["A keyboard-accessible workspace picker shows configured entries and the current workspace and switches to a selected valid board.", "After switching, board state, theme and mutations use the selected workspace without leaking selection, filters or overlays from the old board.", "Background sync follows the active workspace; the old final publish does not stall input and errors remain visible.", "A missing, invalid or unsupported target leaves the previous board open and usable."]
  claimedAt: "2026-10-12T16:29:32Z"
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-12T16:29:32Z"
createdAt: "2026-10-12T15:51:21Z"
updatedAt: "2026-10-12T16:30:53Z"
assignee: "worker-task-72-94e1e002"
---

## Description

Approved task-70 follow-up. Add a searchable workspace picker over the hand-written list, recent-first within the session, with clear current-workspace identity, counts and attention indicators. Switching reloads the selected board and resets old selection, filters, scroll and overlays; mutating actions always target the new project. Only the active workspace has ongoing background sync. Finish the old workspace's publish without blocking the UI, and report failures. A failed target leaves the current board usable. Do not switch during an unfinished edit/prompt. Overview is separate. Kickoff approved by Algorant via Herdra: implement in order 71 → 75. Visual Tasks remain delivered for Algorant's review, with actual screenshots in Sideshow. Keep docs overhaul task-76 paused.


## Delegated preview and ownership

Provide a release-build preview on safe disposable multi-workspace fixtures, including an isolated user workspace list. The normal checkout's just dev must route to the delegated preview without requiring Algorant to modify real user config. If needed, narrowly extend scripts/dev.sh and the existing dev-route helper to carry that isolated config. Do not modify or resume docs-overhaul. Existing small main-branch usage docs may be updated for the verified switcher. New verification tests/harnesses are temporary and removed before delivery. BoardSummary and the all-workspaces overview remain task-74; don't build the overview now.