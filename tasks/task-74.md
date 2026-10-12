---
id: task-74
uid: 73438650-cd27-47f2-a40d-47f690c163b1
type: task
title: "Orchestrator mode: local BoardSummary and session activity overview"
state: todo
priority: "medium"
effort: "large"
blockers: ["task-73"]
references: ["task-70", "task-73"]
relatedFiles: ["tandem/src/app/queries.rs", "tandem/src/app/dto.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/pickers.rs", "tandem/src/tui/input.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/bindings.rs", "tandem/README.md"]
tags: ["tui", "orchestrator"]
accord:
  status: "blocked"
  acceptance: ["One shared app-layer BoardSummary over roster project handles provides local Task-only state counts, current in-progress Task titles/recorded assignees, delivered-needs-triage, explicit Validation, blocked-Accord signals and per-entry errors with clear counting semantics.", "The TUI working-set surface stays compact, labels inactive data local with last-checked times, displays attention only when present and keeps errors inline; it does not claim live agent/process status or remote freshness.", "Summary refresh runs on opening the working-set surface and about every five seconds while visible, without full history/rules loads or remote fetch loops; co-located board writes appear on subsequent local reads.", "Selecting a board from the activity surface uses task-73's existing full-TUI switch boundary and preserves the same editable ephemeral roster.", "Release captures and actual targeted verification are published in Sideshow for Algorant's visual review."]
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Separate root-Task Worker, explicitly Luna/high, only after task-73 is accepted/integrated; plan first.", "Local snapshots only; no inactive-board remote polling, Herdr topology mirroring, live agent-status inference or web work.", "Replace provisional picker counting with one shared query; no competing roster/parser/state system.", "Actual visual delivery in Sideshow B_I5vQctbo0 awaits Algorant's look before merge/completion; no push/release. Temporary checks only."]
  note: "Await task-73 acceptance/integration; approved local BoardSummary/session-activity scope, not frozen redesign."
  updatedAt: "2026-10-12T17:32:34Z"
createdAt: "2026-10-12T15:51:24Z"
updatedAt: "2026-10-12T17:32:34Z"
---
## Approved replacement scope

Algorant approved OefksQbYnFo and Herdra signed off the TUI-first plan. All previous freeze instructions and permanent-config scope are superseded. This root Task follows accepted/integrated task-73 and adds shared BoardSummary plus the compact session working-set activity surface; web remains deferred.

Reuse task-73's in-memory roster/project handles/open-by-path/switch interface. Replace its provisional chooser summary sampling with one app-layer BoardSummary path, reusable by a later closely related web view. Do not add another registry, protocol parser or reduced-board TUI. Detail belongs to each selected board's full existing TUI.

Show normal Board state counts excluding Decisions; current in-progress Task titles/recorded assignees; separately label delivered awaiting triage versus explicit Validation and blocked Accord. These are board records, never live agent status. Last checked is when the local read completed, not remote sync freshness. Inactive-board data is explicitly local. Local paths do not cover remote-only machines' boards.

Read on surface open and around every five seconds while visible. Retain per-entry read errors and keep usable boards visible. No remote fetch loop, expensive full load_read of history/rules every tick, Herdr topology mirror or web implementation. Terse visuals follow approved post OefksQbYnFo; key w/W remain subject to conflicts. Parent verifies native/targeted cases and actual release rendering, publishes in B_I5vQctbo0 and holds delivery for Algorant.