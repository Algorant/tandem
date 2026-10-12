---
id: task-74
uid: 73438650-cd27-47f2-a40d-47f690c163b1
type: task
title: "Orchestrator mode: local BoardSummary and session activity overview"
state: "in-progress"
priority: "medium"
effort: "large"
blockers: ["task-73"]
references: ["task-70", "task-73"]
relatedFiles: ["tandem/src/app/queries.rs", "tandem/src/app/dto.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/pickers.rs", "tandem/src/tui/input.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/bindings.rs", "tandem/README.md", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/board/mod.rs"]
tags: ["tui", "orchestrator"]
accord:
  status: "delivered"
  acceptance: ["One shared app-layer BoardSummary over roster project handles provides local Task-only state counts, current in-progress Task titles/recorded assignees, delivered-needs-triage, explicit Validation, blocked-Accord signals and per-entry errors with clear counting semantics.", "The TUI working-set surface stays compact, labels inactive data local with last-checked times, displays attention only when present and keeps errors inline; it does not claim live agent/process status or remote freshness.", "Summary refresh runs on opening the working-set surface and about every five seconds while visible, without full history/rules loads or remote fetch loops; co-located board writes appear on subsequent local reads.", "Selecting a board from the activity surface uses task-73's existing full-TUI switch boundary and preserves the same editable ephemeral roster.", "Release captures and actual targeted verification are published in Sideshow for Algorant's visual review."]
  claimedAt: "2026-10-12T18:31:05Z"
  deliveredAt: "2026-10-12T19:11:30Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Separate root-Task Worker, explicitly Luna/high, only after task-73 is accepted/integrated; plan first.", "Local snapshots only; no inactive-board remote polling, Herdr topology mirroring, live agent-status inference or web work.", "Replace provisional picker counting with one shared query; no competing roster/parser/state system.", "Actual visual delivery in Sideshow B_I5vQctbo0 awaits Algorant's look before merge/completion; no push/release. Temporary checks only."]
  summary: "Shared local BoardSummary integrated into the existing session chooser with cached asynchronous visible-only refresh, per-board errors and bounded current work/recorded assignees. Parent native/live verification passes; actual release captures published for Algorant's look. Retained clean at 4aac1b4, not merged/completed/pushed. Existing --add-workspace/manual roster contract unchanged; no Herdr discovery, web work or remote polling."
  evidence: ["Sideshow visual review: http://desktop.tail1cefc.ts.net:8228/session/B_I5vQctbo0, post KqgqYFfw2nE; MagicDNS HTTP 200. Actual Herdr ANSI snapshots rendered at 110x32 and 80x24, not mockups.", "Parent final just dev-check, cargo fmt --manifest-path tandem/Cargo.toml -- --check, cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings, and git diff --check passed on 4aac1b4; checkout clean.", "Alpha live fixture counts exactly 6 todo / 4 in-progress / 1 Validation, including Epic/Task/Subtask and excluding Decision/history. Delivered-needs-triage=1, blocked Accord=1; delivered-in-Validation not double-counted as triage and an active blocker alone not counted as blocked Accord.", "Parent changed beta's recorded assignee locally: visible search removed beta when it no longer matched. A subsequent native title update appeared within the refresh interval while an unsubmitted Add-path prompt retained its text and selection.", "Parent observed inactive beta file opens while chooser visible; after confirmed chooser closure inotify observed none during seven seconds. No new remote/discovery calls appear in the shared query path.", "Parent temporarily moved the entire disposable beta repository: summary showed Partial with inline read issue, did not recreate the missing path, and recovered when the unchanged repository was restored. No archived records edited.", "At 80x24 with the long supplied title, actual rendering retained task-2/agent-a and task-3/agent-b, delivered/Validation/blocked signals and local last-check label. Reader job transfer/shutdown reaping reviewed; Worker temporary test verified same thread tracked across replacement and joined.", "Parent selected beta/alpha through the unchanged full-TUI boundary and confirmed the roster remained available. --add-workspace, launch initialization and CLI files are unchanged.", "just dev from normal checkout is routed to safe /tmp/tandem-activity-preview.EsnV4p alpha/beta fixtures without setup or tracked main helper mirrors; main clean. Human activity-layout/temporal judgment pending before integration/completion. Web/task-75 deferred, docs/task-76 paused, no release/push."]
  filesChanged: ["tandem/src/app/queries.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/pickers.rs"]
  updatedAt: "2026-10-12T19:11:30Z"
createdAt: "2026-10-12T15:51:24Z"
updatedAt: "2026-10-12T19:11:30Z"
assignee: "worker-task-74-bf0456b4"
---
## Approved replacement scope

Algorant approved OefksQbYnFo and Herdra signed off the TUI-first plan. All previous freeze instructions and permanent-config scope are superseded. This root Task follows accepted/integrated task-73 and adds shared BoardSummary plus the compact session working-set activity surface; web remains deferred.

Reuse task-73's in-memory roster/project handles/open-by-path/switch interface. Replace its provisional chooser summary sampling with one app-layer BoardSummary path, reusable by a later closely related web view. Do not add another registry, protocol parser or reduced-board TUI. Detail belongs to each selected board's full existing TUI.

Show normal Board state counts excluding Decisions; current in-progress Task titles/recorded assignees; separately label delivered awaiting triage versus explicit Validation and blocked Accord. These are board records, never live agent status. Last checked is when the local read completed, not remote sync freshness. Inactive-board data is explicitly local. Local paths do not cover remote-only machines' boards.

Read on surface open and around every five seconds while visible. Retain per-entry read errors and keep usable boards visible. No remote fetch loop, expensive full load_read of history/rules every tick, Herdr topology mirror or web implementation. Terse visuals follow approved post OefksQbYnFo; key w/W remain subject to conflicts. Parent verifies native/targeted cases and actual release rendering, publishes in B_I5vQctbo0 and holds delivery for Algorant.