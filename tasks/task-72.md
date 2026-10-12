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
  status: "delivered"
  acceptance: ["A keyboard-accessible workspace picker shows configured entries and the current workspace and switches to a selected valid board.", "After switching, board state, theme and mutations use the selected workspace without leaking selection, filters or overlays from the old board.", "Background sync follows the active workspace; the old final publish does not stall input and errors remain visible.", "A missing, invalid or unsupported target leaves the previous board open and usable."]
  claimedAt: "2026-10-12T16:29:32Z"
  deliveredAt: "2026-10-12T16:49:02Z"
  validation: ["$ just dev-check"]
  summary: "Implemented and committed the searchable TUI workspace switcher, isolated release-preview routing, and nonblocking tracked retirement/final-publish for old workspace sync. Ready for parent integration and Algorant's visual review; no Sideshow post or workflow completion was performed."
  evidence: ["Keyboard-accessible searchable workspace switcher shows configured entries and current workspace: `w` opens the picker outside prompts; printable keys filter results, arrows navigate, Enter switches, Esc cancels. Release UI rendered `● beta` as current and showed the configured alpha entry with counts.", "Switch loads selected board/theme and resets old view state: A temporary test switched from a Logs view with an active filter, nonzero detail scroll, and Help overlay to the target's Board; filter, scroll, overlay, and old selection were absent. Release pane showed `beta · beta Workspace` and `Beta unique fixture task`. Each target is prepared and diagnostics checked before replacing the current app.", "Counts and attention indicators avoid per-frame board reads: Counts are sampled when opening the picker, not on frame rendering. The release fixture rendered `! 2 tasks · 1 need attention` for alpha's one validation task.", "Background sync follows active workspace; final publish does not block input and errors remain visible: Switching replaces the active sync handle after immediately signaling the old worker. A tracked retirement job joins it and final-publishes off the event loop; returned failures and panics are alias-qualified and surfaced. Retiring jobs are polled/reaped while the TUI runs and joined after terminal teardown on shutdown.", "Missing, malformed, or unsupported targets preserve the current usable board: A temporary test selected a target with unsupported protocol `99.0.0`; the status reported `Unsupported protocol version`, and the active target root stayed unchanged. Candidate `load_errors` also reject malformed board data instead of treating `TuiApp::load_with` returning Ok as sufficient.", "Preview uses disposable boards and isolated workspace configuration: Fixture root: `/tmp/tandem-task72-preview.gLXD4H`; isolated XDG config home: `/tmp/tandem-task72-preview.gLXD4H/config` (workspace list: `/tmp/tandem-task72-preview.gLXD4H/config/tandem/workspaces.toml`). Alpha and beta have distinct tasks/themes, with an attention item in alpha. Existing `dev-route` now requires this config home and `scripts/dev.sh` applies it only to the routed preview."]
  filesChanged: ["justfile", "scripts/dev.sh", "tandem/README.md", "tandem/src/app/background_sync.rs", "tandem/src/tui/bindings.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/input.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/pickers.rs", "tandem/src/tui/reload.rs", "tandem/src/web.rs"]
  updatedAt: "2026-10-12T16:49:02Z"
createdAt: "2026-10-12T15:51:21Z"
updatedAt: "2026-10-12T16:49:02Z"
assignee: "worker-task-72-94e1e002"
---

## Description

Approved task-70 follow-up. Add a searchable workspace picker over the hand-written list, recent-first within the session, with clear current-workspace identity, counts and attention indicators. Switching reloads the selected board and resets old selection, filters, scroll and overlays; mutating actions always target the new project. Only the active workspace has ongoing background sync. Finish the old workspace's publish without blocking the UI, and report failures. A failed target leaves the current board usable. Do not switch during an unfinished edit/prompt. Overview is separate. Kickoff approved by Algorant via Herdra: implement in order 71 → 75. Visual Tasks remain delivered for Algorant's review, with actual screenshots in Sideshow. Keep docs overhaul task-76 paused.


## Delegated preview and ownership

Provide a release-build preview on safe disposable multi-workspace fixtures, including an isolated user workspace list. The normal checkout's just dev must route to the delegated preview without requiring Algorant to modify real user config. If needed, narrowly extend scripts/dev.sh and the existing dev-route helper to carry that isolated config. Do not modify or resume docs-overhaul. Existing small main-branch usage docs may be updated for the verified switcher. New verification tests/harnesses are temporary and removed before delivery. BoardSummary and the all-workspaces overview remain task-74; don't build the overview now.