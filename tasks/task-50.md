---
id: task-50
uid: ef34cbd8-c699-4582-a2af-2ae1ed59bd51
type: task
kind: papercut
title: "Event envelope in code contradicts protocol spec (summary vs data)"
state: todo
priority: "low"
relatedFiles: ["protocol/README.md", "protocol/plan/spec.md", "tandem/src/protocol/event.rs", "tandem/src/project/events.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/app/accord.rs", "tandem/src/app/rules.rs", "tandem/src/app/links.rs", "tandem/src/app/review.rs"]
tags: ["protocol", "events"]
accord:
  status: "ready"
  acceptance: ["The protocol spec and the executable event envelope agree on whether `summary` and `data` are required, with a test asserting the chosen contract."]
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Plan first, Lua/high; new writes must follow agreed normative contract, not simply rewrite docs to match the bug.", "No history/archived-ledger edits or fabricated full-body event payloads. Preserve legitimate existing history; concrete reader transition belongs in plan.", "No protocol version bump, source push/release or general event framework without separate approval. Temporary checks only."]
  updatedAt: "2026-10-13T13:54:46Z"
createdAt: "2026-09-30T03:41:03Z"
updatedAt: "2026-10-13T13:54:46Z"
effort: "medium"
---
## Approved batch B, before task-64

Current 0.17 code really requires summary and makes data optional, while normative README/spec requires ts/event/id/actor/seq plus event-specific structured data and no required free-text summary. Align the executable contract, writers/readers and normative text. Proposed new-write envelope has required ts/event/id/actor/seq/data; summary is not required. Keep audit payload minimal; no full Markdown bodies.

Existing audit history includes summary-only lines and must not be rewritten or silently lost. Your plan must state the exact reader transition and preservation checks before implementation; don't invent a speculative compatibility layer. Preserve current actor/sequence ownership and allocation. Own event protocol/project writer/reader and necessary app callers; task-64 starts only after this lands so event-boundary edits do not overlap. Docs prose waits for this verified contract.

Fresh Luna/high Worker, plan first. Temporary no-summary/new-data/old-prefix/history and event-kind checks; native suite/fmt/strict clippy. Escalate any larger-than-request machinery before approval. No push/release.