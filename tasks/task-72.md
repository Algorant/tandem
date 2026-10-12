---
id: task-72
uid: 2906d301-e968-4665-bf36-68ae974376b0
type: task
title: "Orchestrator mode: TUI workspace switcher"
state: "todo"
priority: "medium"
effort: "medium"
blockers: ["task-71"]
references: ["task-70", "task-71", "task-73"]
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/state.rs", "tandem/src/app/background_sync.rs", "tandem/src/tui/pickers.rs", "tandem/src/project/workspaces.rs", "scripts/dev.sh", "justfile", "tandem/README.md", "tandem/src/tui/input.rs", "tandem/src/tui/bindings.rs"]
tags: ["tui", "orchestrator", "keyboard"]
accord:
  status: "ready"
  acceptance: ["A keyboard-accessible workspace picker shows configured entries and the current workspace and switches to a selected valid board.", "After switching, board state, theme and mutations use the selected workspace without leaking selection, filters or overlays from the old board.", "Background sync follows the active workspace; the old final publish does not stall input and errors remain visible.", "A missing, invalid or unsupported target leaves the previous board open and usable."]
  claimedAt: "2026-10-12T16:29:32Z"
  deliveredAt: "2026-10-12T17:09:52Z"
  validation: ["$ just dev-check"]
  constraints: ["Superseded legacy implementation: no further work in its retained Worker; preserve until task-73's independently verified reuse checkpoint.", "After verified transfer, parent is authorized to worker_discard the exact old attempt and cancel this Task as superseded, not ship/accept it."]
  summary: "Searchable TUI workspace switcher implemented and independently verified; ready for Algorant's requested visual review. Actual release-TUI ANSI snapshot images published in the existing orchestrator Sideshow session. Worker branch retained at 4f6d5e7, not merged, pushed or completed. Normal just dev is routed to isolated alpha/beta/old fixtures; temporary parent justfile/scripts/dev.sh helper mirrors enable the three-line preview route pending acceptance."
  evidence: ["Sideshow review: http://desktop.tail1cefc.ts.net:8228/session/B_I5vQctbo0, post WVRSpB9l9Mw; bounded MagicDNS check returned HTTP 200. Images render actual Herdr ANSI captures at 110x32, not mockups.", "Parent ran just dev-check and cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings on final Worker code: both passed; git diff --check passed.", "Parent live Herdr verification: w picker/search beta/Enter changed the board title, unique task and theme; historical old remained unavailable and its refusal led the footer at 110 columns, preserving beta.", "Parent post-switch TUI claim set only beta task-1's assignee to parent-review-agent; alpha task-1 remained todo/ready with no assignee, proving mutations retarget correctly.", "Reviewed fresh candidate load/state reset and tracked nonblocking old-sync retirement/final publish; malformed/historical/unsupported candidate paths preserve the current board. Temporary tests removed and Worker checkout clean.", "Release preview: just dev from normal checkout requires no extra setup and uses /tmp/tandem-task72-preview.gLXD4H/config with safe disposable boards; real user workspace list and real boards untouched. Human layout/usability and temporal terminal judgment remain pending."]
  filesChanged: ["justfile", "scripts/dev.sh", "tandem/README.md", "tandem/src/app/background_sync.rs", "tandem/src/tui/bindings.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/input.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/pickers.rs", "tandem/src/tui/reload.rs", "tandem/src/web.rs"]
  note: "discarded: Authorized by Algorant/Herdra after verified reuse transfer. Fresh task-73 checkpoint b0d144edd2cd30d73a17a1c77ef1ca5f8fd02c61 is clean and its entire source tree exactly matches old 4f6d5e7; /tmp/tandem-task72-reuse.bundle independently preserves old commits with prerequisite main 1bfc31c. This legacy permanent-list Worker is throwaway, superseded by fresh ephemeral-session implementation; no merge/acceptance."
  updatedAt: "2026-10-12T17:35:33Z"
createdAt: "2026-10-12T15:51:21Z"
updatedAt: "2026-10-12T17:35:33Z"
---
## Superseded legacy attempt pending safe transfer

Algorant approved ephemeral TUI session working set OefksQbYnFo and explicitly called this old Worker throwaway. Prior freeze and permanent-list implementation instructions no longer apply; do not continue this Worker. The fresh implementation is root task-73, followed by BoardSummary/activity task-74. Web task-75 is deferred.

Retained old Worker: worker-task-72-94e1e002, clean checkpoint 4f6d5e7587497d64f6ed13251141f357d6291371. Portable git bundle /tmp/tandem-task72-reuse.bundle preserves its branch with main prerequisite 1bfc31c. Reuse full-TUI candidate loading/reset, prompt safeguards, historical/error refusal, correctly retargeted mutations and tracked nonblocking sync retirement with visible alias-qualified errors. Drop its permanent-list source and TOML-seeded preview.

Parent will first verify the fresh Worker's committed reuse checkpoint and independent bundle, then discard this exact retained attempt through worker_discard and cancel this Task as superseded by task-73. No merge/worker_finish of this legacy attempt, and no force cleanup on a Worktrunk refusal. Report the disposal outcome to Herdra.