---
id: task-78
uid: 00fe6b0b-b177-4ae7-830a-1ac94adc4f67
type: task
kind: "papercut"
title: "TUI archive validation rejects logs produced by complete without --note"
priority: "low"
effort: "small"
references: ["task-73"]
relatedFiles: ["tandem/src/protocol/diagnostic.rs", "tandem/src/protocol/workflow.rs", "protocol/README.md", "tandem/src/tui/state.rs"]
tags: ["protocol", "tui"]
accord:
  status: "accepted"
  acceptance: ["An otherwise valid archived Task produced by native complete without --note is not reported invalid solely because resolution.note is absent; protocol guidance and executable diagnostics agree on optionality.", "A disposable board containing such a log loads in the normal full TUI and through the session switcher, while genuinely invalid archive metadata remains diagnosed."]
  claimedAt: "2026-10-12T18:00:02Z"
  deliveredAt: "2026-10-12T18:04:26Z"
  validation: ["$ just dev-check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Keep resolution.note optional as already accepted by complete/ResolutionRecord; retain genuinely invalid archive diagnostics and existing cancel/block note requirements.", "No archived-record edits, TUI-side bypass or permanent new tests/fixtures; temporary verification only.", "No push/release. Keep task-73's implementation checkout untouched; parent owns its rebase and visual verification."]
  summary: "Removed the archived Task diagnostic requiring resolution.note and clarified in protocol guidance that resolution.note/reviewer are optional. Kept archivedAt and outcome validation. Committed as 3ade5cd; checkout is clean."
  evidence: ["An otherwise valid archived Task without resolution.note is accepted; guidance and executable diagnostics agree on optionality.: The native complete workflow emitted resolution.outcome: completed with no note; list all returned no warnings. The temporary direct diagnostic check likewise accepted the no-note archive. Protocol README states note/reviewer are optional.", "Genuinely invalid archive metadata remains diagnosed.: The temporary direct diagnostic check confirmed invalid resolution.outcome and absent archivedAt each produced an error.", "No archived-record edits or TUI-side bypass; no permanent tests/fixtures.: Only protocol/README guidance and protocol metadata diagnostics changed; the temporary test was removed. Validation used disposable /tmp boards."]
  filesChanged: ["tandem/src/protocol/diagnostic.rs", "protocol/README.md"]
  updatedAt: "2026-10-12T18:04:27Z"
createdAt: "2026-10-12T17:59:04Z"
updatedAt: "2026-10-12T18:04:27Z"
assignee: "worker-task-78-c497e5c8"
archivedAt: "2026-10-12T18:04:27Z"
resolution:
  outcome: "completed"
---

## Description

Observed during independent live task-73 review on a disposable Git board. Native claim → deliver → complete (no --note) creates a log with resolution.outcome completed but no resolution.note. list --scope all accepts it with no warnings; complete --help makes --note optional; protocol::workflow::ResolutionRecord.note is Option<String>. However protocol::diagnostic::metadata_diagnostics requires resolution.note on every archived Task. Full TUI load captures that error, and the new safe session switcher refuses the board. Reproduction log: /tmp/tandem-session-preview.UFMtuE/beta/.tandem/logs/task-2.md. Do not edit archived records to work around this. Resolve the diagnostic/contract mismatch narrowly, with temporary checks only (no permanent new tests). Herdra authorized implementation under Algorant's approved plan. Align the protocol-owned diagnostic with the existing optional-note archive contract, with no archived-record edits or TUI-side workaround. Parent will integrate this fix into main, rebase task-73 onto it and verify the actual session-switch path before task-73's visual delivery.
