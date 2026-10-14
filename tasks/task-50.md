---
id: task-50
uid: ef34cbd8-c699-4582-a2af-2ae1ed59bd51
type: task
kind: papercut
title: "Event envelope in code contradicts protocol spec (summary vs data)"
state: "in-progress"
priority: "low"
relatedFiles: ["protocol/README.md", "protocol/plan/spec.md", "tandem/src/protocol/event.rs", "tandem/src/project/events.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/app/accord.rs", "tandem/src/app/rules.rs", "tandem/src/app/links.rs", "tandem/src/app/review.rs"]
tags: ["protocol", "events"]
accord:
  status: "delivered"
  acceptance: ["The protocol spec and the executable event envelope agree on whether `summary` and `data` are required, with a test asserting the chosen contract."]
  claimedAt: "2026-10-13T13:55:03Z"
  deliveredAt: "2026-10-14T01:50:48Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Plan first, Lua/high; new writes must follow agreed normative contract, not simply rewrite docs to match the bug.", "No history/archived-ledger edits or fabricated full-body event payloads. Preserve legitimate existing history; concrete reader transition belongs in plan.", "No protocol version bump, source push/release or general event framework without separate approval. Temporary checks only."]
  summary: "Structured event envelope and caller/reader contract independently verified at d49be5e. Newwrites require ts/event/id/actor/seq/data, omit summary; legitimate summary-only and summary+data history remains readable without rewrite. Parent native checks and release TUI Logs data-label render pass; accept technical fix without further human review. No push/release."
  evidence: ["Parent reviewed full11-file diff198insertions/152deletions: requireddata object/no serializedsummary; existing actor/lock/sequence authority and release disposition/link payloads retained; no framework/version/prose/history changes.", "Parent independently ran just dev-check,fmt,strictclippy anddiffcheck: all passed. 337unit tests and76integration tests in existing project suites passed; no new permanent test functions/fixtures, existing contract tests updated.", "Parent disposable native CLI check copied genuine256-record ledger unchanged; SHA2564bcd1ba3f9a01f413b9b97dcd55bdceb820691c1a8eeed3b7795ed9aaf139e89 identical after newadd/update/claim/deliver/review/complete. Six newevents have EXACT ts/event/id/actor/seq/data keys, objectdata, contiguousseq; correct title/changed-fields metadata.", "Existing reader tests verify all3 history shapes and reject malformedJSON/missingboth/nonobjectdata; modified existing append test verifies byte-identical mixedsummary+data prefix andnextseq. Worker genuine62202byte mixed-history append smoke verifiedseq258, fixture removed.", "Parent releaseTUI pane wC:pE on disposableboard: Logs1 archive, real Eventtimeline with task.created Auditcheck and task.updated Updated:tags, lifecycle eventtypes visible; no parse warnings or summaryfield failure. ANSI evidence /tmp/task50-parent-timeline.ansi retained. Parent pane/helper/fixture cleaned; real board/history untouched.", "No acknowledged blocker; context1 authorizes parent completion of approved verifiable technical work. worker_finish integrates/accepts/cleans exactsource only; no push/deploy/release. task64 may start after integration against finalized append_event(project,event,id,data)."]
  filesChanged: ["tandem/src/protocol/event.rs", "tandem/src/project/events.rs", "tandem/src/project/mod.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/app/accord.rs", "tandem/src/app/decisions.rs", "tandem/src/app/rules.rs", "tandem/src/app/links.rs", "tandem/src/app/review.rs", "tandem/src/tui/logs.rs"]
  updatedAt: "2026-10-14T01:50:48Z"
createdAt: "2026-09-30T03:41:03Z"
updatedAt: "2026-10-14T01:50:48Z"
effort: "medium"
assignee: "worker-task-50-aab3bafd"
---
## Approved batch B, before task-64

Current 0.17 code requires summary and makes data optional; normative README/spec requires ts/event/id/actor/seq/data, without required free-text summary. Align executable envelope, writers/readers/app callers and Logs display; minimal structured audit payloads, no full Markdown bodies. Existing actor/sequence ownership unchanged.

## Concrete plan approved 2026-10-14

Parent recovered the actual revised plan from the exact retained session after worker_report failed on second-plan guard (captured as ~/.pi task-437). Implementation is explicitly approved; no further plan gate. New writes require data and omit summary. Read-only historical compatibility must preserve summary-only AND legitimate summary+data records (current link/release writers emit both), plus new data-only records; no synthesized payload/history rewrite. Temporary genuine-ledger byte/prefix/read-count/next-sequence/new-shape checks, mixed-history regression, native suite/fmt/strictclippy. Logs renders preserved summary when present, otherwise concise event/data label. Only existing event names/callers; no decision.withdrawn addition, global-ledger transition framework or version bump. Current obsolete global-ledger exclusion remains.

Owned scope: protocol event type, project writer/reader, necessary app support/tasks/accord/decisions/rules/review/links callers and TUI Logs projection. Normative text already correct; do not rewrite it to match bug. No source overlaps: task64 waits until this is verified/integrated, then task63. Temporary added checks removed before delivery; no permanent new fixtures/tests. Parent approval sent to same worker-task-50-aab3bafd and working state independently confirmed. No push/release.