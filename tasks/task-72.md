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
  status: "blocked"
  acceptance: ["A keyboard-accessible workspace picker shows configured entries and the current workspace and switches to a selected valid board.", "After switching, board state, theme and mutations use the selected workspace without leaking selection, filters or overlays from the old board.", "Background sync follows the active workspace; the old final publish does not stall input and errors remain visible.", "A missing, invalid or unsupported target leaves the previous board open and usable."]
  claimedAt: "2026-10-12T16:29:32Z"
  deliveredAt: "2026-10-12T17:09:52Z"
  validation: ["$ just dev-check"]
  constraints: ["Frozen: revised ephemeral-workspace plumbing/functions require Algorant's approval before any further code, mockups, integration or push. Preserve current work; no discards/rollbacks."]
  summary: "Searchable TUI workspace switcher implemented and independently verified; ready for Algorant's requested visual review. Actual release-TUI ANSI snapshot images published in the existing orchestrator Sideshow session. Worker branch retained at 4f6d5e7, not merged, pushed or completed. Normal just dev is routed to isolated alpha/beta/old fixtures; temporary parent justfile/scripts/dev.sh helper mirrors enable the three-line preview route pending acceptance."
  evidence: ["Sideshow review: http://desktop.tail1cefc.ts.net:8228/session/B_I5vQctbo0, post WVRSpB9l9Mw; bounded MagicDNS check returned HTTP 200. Images render actual Herdr ANSI captures at 110x32, not mockups.", "Parent ran just dev-check and cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings on final Worker code: both passed; git diff --check passed.", "Parent live Herdr verification: w picker/search beta/Enter changed the board title, unique task and theme; historical old remained unavailable and its refusal led the footer at 110 columns, preserving beta.", "Parent post-switch TUI claim set only beta task-1's assignee to parent-review-agent; alpha task-1 remained todo/ready with no assignee, proving mutations retarget correctly.", "Reviewed fresh candidate load/state reset and tracked nonblocking old-sync retirement/final publish; malformed/historical/unsupported candidate paths preserve the current board. Temporary tests removed and Worker checkout clean.", "Release preview: just dev from normal checkout requires no extra setup and uses /tmp/tandem-task72-preview.gLXD4H/config with safe disposable boards; real user workspace list and real boards untouched. Human layout/usability and temporal terminal judgment remain pending."]
  filesChanged: ["justfile", "scripts/dev.sh", "tandem/README.md", "tandem/src/app/background_sync.rs", "tandem/src/tui/bindings.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/input.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/pickers.rs", "tandem/src/tui/reload.rs", "tandem/src/web.rs"]
  note: "Algorant froze orchestrator-mode implementation pending approved ad-hoc ephemeral-workspace plumbing; permanent workspaces.toml is no longer the requirement. Preserve existing work unchanged; no new code/mockups/merges/pushes/discards/rollbacks."
  updatedAt: "2026-10-12T17:18:01Z"
createdAt: "2026-10-12T15:51:21Z"
updatedAt: "2026-10-12T17:18:01Z"
assignee: "worker-task-72-94e1e002"
---
## Frozen by Algorant via Herdra

Do not implement, merge, push, create implementation Workers, produce mockups, discard, clean up or roll back existing work until Algorant approves revised plumbing/functions. Preserve all commits, checkouts and current preview state. New requirement: no dedicated permanent workspaces.toml; enabled/viewable workspaces must be ad-hoc, ephemeral and easy to change during a session. Discuss session roster, on-demand native Herdr project discovery and a repo picker first; no UI/button design yet. w/W remain tentative subject to conflicts. This direction overrides the earlier permanent-list implementation scope below, which is retained as history, not renewed approval.

Task-71 already landed as 1bfc31c and is archived accepted; its code is preserved, not rolled back. Task-72's retained unmerged Worker is clean at 4f6d5e7. Main retains temporary justfile/scripts/dev.sh preview helper mirrors; do not restore them while the no-rollback hold applies.

## Previous scope (on hold)


## Description

Approved task-70 follow-up. Add a searchable workspace picker over the hand-written list, recent-first within the session, with clear current-workspace identity, counts and attention indicators. Switching reloads the selected board and resets old selection, filters, scroll and overlays; mutating actions always target the new project. Only the active workspace has ongoing background sync. Finish the old workspace's publish without blocking the UI, and report failures. A failed target leaves the current board usable. Do not switch during an unfinished edit/prompt. Overview is separate. Kickoff approved by Algorant via Herdra: implement in order 71 → 75. Visual Tasks remain delivered for Algorant's review, with actual screenshots in Sideshow. Keep docs overhaul task-76 paused.


## Delegated preview and ownership

Provide a release-build preview on safe disposable multi-workspace fixtures, including an isolated user workspace list. The normal checkout's just dev must route to the delegated preview without requiring Algorant to modify real user config. If needed, narrowly extend scripts/dev.sh and the existing dev-route helper to carry that isolated config. Do not modify or resume docs-overhaul. Existing small main-branch usage docs may be updated for the verified switcher. New verification tests/harnesses are temporary and removed before delivery. BoardSummary and the all-workspaces overview remain task-74; don't build the overview now.