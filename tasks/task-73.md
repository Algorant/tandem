---
id: task-73
uid: f1f67c9a-042d-4bd2-9e53-11e9579a634b
type: task
title: "Orchestrator mode: ephemeral TUI working set and full-board switching"
state: "in-progress"
priority: "medium"
effort: "large"
blockers: ["task-71"]
references: ["task-70", "task-71", "task-72"]
relatedFiles: ["tandem/src/app/project.rs", "tandem/src/project/mod.rs", "tandem/src/project/workspaces.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/input.rs", "tandem/src/tui/pickers.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/bindings.rs", "tandem/src/tui/board/mod.rs", "tandem/src/app/background_sync.rs", "tandem/src/web.rs", "tandem/Cargo.toml", "tandem/Cargo.lock", "scripts/dev.sh", "justfile", "tandem/README.md"]
tags: ["tui", "orchestrator", "keyboard"]
accord:
  status: "rework"
  acceptance: ["The TUI owns one in-memory session working set seeded with the launch-directory board; it is retained across switches and lost at exit, with no permanent roster file read, write or fallback.", "The chooser adds existing boards by path using open_at and canonical resolved-board-root dedupe (including linked worktrees), reports invalid paths without changing the current board/set, removes only session membership, and prevents removing the current board.", "Selecting a board switches in place to the complete existing Board/Logs/Rules/Decisions TUI with normal detail/actions/filter/editor support; candidate refusal and prompt safety preserve the current usable board.", "Mutations and ongoing background sync follow the active board; old final publishes retire off the input loop with tracked, alias-qualified errors and no leaked per-board view state.", "The obsolete TOML workspace loader/startup wiring/docs and unused TOML dependency are removed, while open_at, canonical-path handling and shared theme-config path behaviour remain.", "Release preview uses disposable boards and a session-only seed; actual TUI captures and verification evidence are published for Algorant's review."]
  claimedAt: "2026-10-12T17:33:03Z"
  deliveredAt: "2026-10-12T17:47:29Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Fresh root-Task Worker only, explicitly openai-codex/gpt-6-luna / high; report plan before editing.", "Reuse checkpoint must be committed and parent-verified before disposal of task-72's old Worker; stop for owner direction at that boundary.", "No permanent workspaces.toml, config-file fallback, auto-add registry, Herdr topology mirroring, remote polling of inactive boards or web feature implementation.", "No real-board or user-config mutations for validation; temporary checks only, removed before ready delivery.", "Visible delivery goes to Sideshow B_I5vQctbo0 and awaits Algorant's look before integration/completion; no push/release. Docs-overhaul remains paused."]
  summary: "Implemented and committed the ephemeral TUI session roster, add/remove-by-path and full-board switching, removal of the persistent TOML loader/dependency/docs, and invocation-only preview seeds. Final implementation commit: 49bcc73; checkout clean. Ready for parent release-render verification and Sideshow review."
  evidence: ["Roster is one ephemeral in-memory set seeded with launch board; no permanent roster read/write/fallback.: `app::project::SessionRoster` owns the handles; TUI replacement candidates share the same `Arc<Mutex<SessionRoster>>`. Startup `--add-workspace` paths feed this same roster. Removed the TOML loader/module, `workspaces.toml` documentation/wiring, and unused `toml` dependency. Existing user files were not accessed or modified.", "Add by path, canonical deduplication, safe invalid-path handling, and inactive-only removal.: Roster additions use canonicalizing `app::project::open_at` and retain its discovered TandemProject handle. Temporary test observed symlink and linked-worktree aliases reuse one root; nonexistent paths fail before insertion; current removal is rejected while inactive removal succeeds. Chooser removal is scoped to session membership only.", "Selecting a board preserves complete TUI capability and safe candidate refusal while avoiding stale per-board view state.: Chooser selection builds a fresh TuiApp through the normal loader using the roster's existing project handle, rejects historical and failed candidate loads before replacing the active TUI, and shares the roster pointer across successful replacement. The imported task-72 fresh-state, prompt-safety, footer, and refusal behavior remains in the implementation; parent visual verification is pending.", "Retired-board background sync does not block input and errors remain attributable.: Preserved imported tracked retirement jobs and alias-qualified periodic/final-publish error notes; active-board changes start a new background sync and retire the old one off the input loop.", "Safe multi-board release preview is invocation-only and default single-board behavior remains.: Dev route now carries a seed-board path and invokes `tandem tui --add-workspace PATH`; no config-home roster file is checked or written. The non-routed `just dev` path remains the existing disposable single-board flow. Two disposable Git fixtures were created for parent capture."]
  filesChanged: ["justfile", "scripts/dev.sh", "tandem/Cargo.lock", "tandem/Cargo.toml", "tandem/README.md", "tandem/src/app/project.rs", "tandem/src/cli/commands.rs", "tandem/src/cli/mod.rs", "tandem/src/cli/model.rs", "tandem/src/main.rs", "tandem/src/project/mod.rs", "tandem/src/project/workspaces.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/pickers.rs", "tandem/src/tui/reload.rs"]
  note: "Reviewed 49bcc73; session-only replacement and preserved switching are on direction. Please correct these bounded issues before actual capture: (1) New 'Cannot add board', bad CLI-seed, and current-removal-refused statuses trail long footer hints; priority_footer_status only knows old switch/sync errors. Show add/remove feedback/errors inside the chooser or lead the footer so they are readable at 80–110 columns while it remains open. Don't reset away add-success/duplicate feedback by immediately calling sta"
  updatedAt: "2026-10-12T17:47:30Z"
createdAt: "2026-10-12T15:51:22Z"
updatedAt: "2026-10-12T17:47:30Z"
assignee: "worker-task-73-0c96df57"
---
## Approved direction

Algorant approved proposed post OefksQbYnFo; Herdra signed off the implementation plan. This replaces the former web-selector scope and all freeze instructions. Implement the TUI session working set, add/remove by path and full existing TUI switching as one cohesive root-Task outcome. Web is deferred to task-75. No permanent workspace-list requirement.

## Roster and boundaries

One session roster lives outside the replaceable per-board view state; seed current board at launch. Add by local path through existing open_at/discovery/protocol checks, resolve canonical board root and dedupe symlinks/linked worktrees. Remove only an inactive entry; current must be switched away from first. Exit forgets the set. No mandatory CLI seed options in the first slice; no filesystem scanning or Herdr discovery system. Existing board opening may recover a board on an explicit user add, but inactive roster reads must not launch a remote polling loop.

Preserve full current TUI capabilities on selection, using task-72 fresh candidate/state-reset/prompt-safety/historical-refusal logic. Local Task-only counts/attention may remain in the chooser initially; task-74 replaces provisional sampling with a single shared BoardSummary and compact current-activity surface. Never imply live agent/process status from recorded assignees or Accord state. Delivered and Validation stay distinct.

## Remove old configuration path

Remove task-71 workspaces.toml loader, startup list wiring/documentation and TOML-only dependency if unused. Do not keep the file as a fallback or seed; do not delete/edit any user's existing file. Retain open_at, canonical root validation/dedupe and shared user-config/theme directory handling.

## Authorized reuse checkpoint

A fresh Worker must inspect and plan before editing. Reuse source checkpoint: 4f6d5e7587497d64f6ed13251141f357d6291371; portable git bundle /tmp/tandem-task72-reuse.bundle contains the old Worker branch with prerequisite main commit 1bfc31c. Parent verified bundle and old clean checkout. New Worker takes/imports useful switching, refusal, mutation-retargeting, footer visibility and tracked background-sync retirement code into its own checkout, commits a transfer checkpoint, verifies independent bundle availability, and stops for owner direction. Parent verifies it, then discards worker-task-72-94e1e002 and cancels legacy task-72 as superseded. Do not touch the old checkout, parent target, workflow or cleanup yourself.

Drop old permanent-list diagnostics/source ownership and TOML-based preview seeding. Existing web.rs BackgroundSync caller adjustment is compilation maintenance only, not permission for a web selector/server/overview. One authoritative source of roster state.

## Validation and preview

Use temporary disposable checks for launch/exit lifetime, add/remove/current guard, duplicate and linked-worktree paths, full views, mutation isolation, theme/filter/selection reset, prompt safeguards, malformed/unsupported/historical rejection, final-publish responsiveness/errors and zero permanent-roster file interaction. Provide safe just dev preview via invocation/session-only fixtures; no real config or real board changes. Parent independently verifies release rendering and publishes actual captures in existing Sideshow session. Keep docs-overhaul task-76/branch parked.