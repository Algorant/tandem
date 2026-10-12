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
  deliveredAt: "2026-10-12T18:47:18Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Separate root-Task Worker, explicitly Luna/high, only after task-73 is accepted/integrated; plan first.", "Local snapshots only; no inactive-board remote polling, Herdr topology mirroring, live agent-status inference or web work.", "Replace provisional picker counting with one shared query; no competing roster/parser/state system.", "Actual visual delivery in Sideshow B_I5vQctbo0 awaits Algorant's look before merge/completion; no push/release. Temporary checks only."]
  summary: "Implemented shared local BoardSummary and integrated it into the existing w/session-board chooser with cached asynchronous refresh and bounded activity. Committed as 1c882f3; checkout is clean. No CLI --add-workspace contract, launch-board behavior, web feature, or external launcher was changed. Parent owns live UI/local-refresh verification and Sideshow delivery before integration/Algorant review."
  evidence: ["Shared app-layer projection over existing roster handles, strict Task-only counts and distinct attention signals.: session_board_summaries consumes SessionBoard handles and delegates to board_summary, which reads only native active Board Task documents. Exact todo/in-progress/validation state counts exclude Decisions/logs; delivered-needs-triage is in-progress+delivered only; explicit Validation and blocked Accord are separate, overlapping signals. Current-work details are capped at four and include title/recorded assignee.", "Existing chooser is compact and explicitly local, with per-entry errors and refresh on open/about every five seconds.: The existing w chooser shows cached counts, selected-board local snapshot age, bounded current-work and attention details. Read errors are visible inline and partial results do not become healthy zero. Summary reads run in one tracked background batch, outside the input path and without holding the roster lock; refresh scheduling is active only while the chooser is visible.", "Selection and ephemeral roster behavior remain on task-73's full-TUI boundary; no CLI contract changes.: Chooser selection still resolves the selected path against the same SessionRoster and invokes switch_workspace/TuiApp::load_with. Ctrl-A/Ctrl-D and path/search/feedback code remain intact. Inspection confirmed --add-workspace remains repeatable path input routed through the existing add_path path; no CLI files changed.", "Exact accepted fixture semantics.: The temporary targeted check passed Alpha expected counts (todo=6, in-progress=4, validation=1), delivered task-3 only in the in-progress cohort, blocked task-5, and four current Tasks; Beta showed one in-progress Task with assignee agent-z. Validation-state delivered task-4 did not add to triage.", "Release handoff and validation readiness.: Commit 1c882f3 contains only the three scoped implementation files; git status was clean after commit. Assigned native validation commands all passed. Parent's required actual release rendering and live local-write/error-path checks remain outstanding by design."]
  filesChanged: ["tandem/src/app/queries.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/pickers.rs"]
  updatedAt: "2026-10-12T18:47:18Z"
createdAt: "2026-10-12T15:51:24Z"
updatedAt: "2026-10-12T18:47:18Z"
assignee: "worker-task-74-bf0456b4"
---
## Approved replacement scope

Algorant approved OefksQbYnFo and Herdra signed off the TUI-first plan. All previous freeze instructions and permanent-config scope are superseded. This root Task follows accepted/integrated task-73 and adds shared BoardSummary plus the compact session working-set activity surface; web remains deferred.

Reuse task-73's in-memory roster/project handles/open-by-path/switch interface. Replace its provisional chooser summary sampling with one app-layer BoardSummary path, reusable by a later closely related web view. Do not add another registry, protocol parser or reduced-board TUI. Detail belongs to each selected board's full existing TUI.

Show normal Board state counts excluding Decisions; current in-progress Task titles/recorded assignees; separately label delivered awaiting triage versus explicit Validation and blocked Accord. These are board records, never live agent status. Last checked is when the local read completed, not remote sync freshness. Inactive-board data is explicitly local. Local paths do not cover remote-only machines' boards.

Read on surface open and around every five seconds while visible. Retain per-entry read errors and keep usable boards visible. No remote fetch loop, expensive full load_read of history/rules every tick, Herdr topology mirror or web implementation. Terse visuals follow approved post OefksQbYnFo; key w/W remain subject to conflicts. Parent verifies native/targeted cases and actual release rendering, publishes in B_I5vQctbo0 and holds delivery for Algorant.