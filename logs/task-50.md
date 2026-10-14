---
id: task-50
uid: ef34cbd8-c699-4582-a2af-2ae1ed59bd51
type: task
kind: papercut
title: "Event envelope in code contradicts protocol spec (summary vs data)"
priority: "low"
relatedFiles: ["protocol/README.md", "protocol/plan/spec.md", "tandem/src/protocol/event.rs", "tandem/src/project/events.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/app/accord.rs", "tandem/src/app/rules.rs", "tandem/src/app/links.rs", "tandem/src/app/review.rs"]
tags: ["protocol", "events"]
accord:
  status: "accepted"
  acceptance: ["The protocol spec and the executable event envelope agree on whether `summary` and `data` are required, with a test asserting the chosen contract."]
  claimedAt: "2026-10-13T13:55:03Z"
  deliveredAt: "2026-10-14T01:52:27Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Plan first, Lua/high; new writes must follow agreed normative contract, not simply rewrite docs to match the bug.", "No history/archived-ledger edits or fabricated full-body event payloads. Preserve legitimate existing history; concrete reader transition belongs in plan.", "No protocol version bump, source push/release or general event framework without separate approval. Temporary checks only."]
  summary: "Refreshed readiness against the parent’s updated task scope. Clean source remains d49be5e with no code changes: new writes require structured data and omit summary; historical summary-only and summary+data records remain readable alongside data-only writes; actor/sequence ownership is unchanged. Parent independently passed review and native checks; no integration, push, or deployment performed."
  evidence: ["Current task scope is implemented without further changes.: Parent feedback confirms the refreshed scope matches d49be5e: mandatory `data`, no newly written `summary`, historical summary-only and summary+data read support, and preserved actor/sequence authority.", "Ready source is clean and independently verified.: No files changed since d49be5e. Parent reports independent review and native checks passed; all temporary parent fixtures/panes were cleaned."]
  filesChanged: ["tandem/src/protocol/event.rs", "tandem/src/project/events.rs", "tandem/src/project/mod.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/app/accord.rs", "tandem/src/app/decisions.rs", "tandem/src/app/rules.rs", "tandem/src/app/review.rs", "tandem/src/app/links.rs", "tandem/src/tui/logs.rs"]
  updatedAt: "2026-10-14T01:52:29Z"
createdAt: "2026-09-30T03:41:03Z"
updatedAt: "2026-10-14T01:52:29Z"
effort: "medium"
assignee: "worker-task-50-aab3bafd"
archivedAt: "2026-10-14T01:52:29Z"
resolution:
  outcome: "completed"
---
## Approved batch B, before task-64

Current 0.17 code requires summary and makes data optional; normative README/spec requires ts/event/id/actor/seq/data, without required free-text summary. Align executable envelope, writers/readers/app callers and Logs display; minimal structured audit payloads, no full Markdown bodies. Existing actor/sequence ownership unchanged.

## Concrete plan approved 2026-10-14

Parent recovered the actual revised plan from the exact retained session after worker_report failed on second-plan guard (captured as ~/.pi task-437). Implementation is explicitly approved; no further plan gate. New writes require data and omit summary. Read-only historical compatibility must preserve summary-only AND legitimate summary+data records (current link/release writers emit both), plus new data-only records; no synthesized payload/history rewrite. Temporary genuine-ledger byte/prefix/read-count/next-sequence/new-shape checks, mixed-history regression, native suite/fmt/strictclippy. Logs renders preserved summary when present, otherwise concise event/data label. Only existing event names/callers; no decision.withdrawn addition, global-ledger transition framework or version bump. Current obsolete global-ledger exclusion remains.

Owned scope: protocol event type, project writer/reader, necessary app support/tasks/accord/decisions/rules/review/links callers and TUI Logs projection. Normative text already correct; do not rewrite it to match bug. No source overlaps: task64 waits until this is verified/integrated, then task63. Temporary added checks removed before delivery; no permanent new fixtures/tests. Parent approval sent to same worker-task-50-aab3bafd and working state independently confirmed. No push/release.