---
id: task-64
uid: b7218abb-725a-482e-8081-450b4e71173c
type: task
kind: "papercut"
title: "Refused mutations must not leave a written record"
state: todo
priority: "low"
references: ["Algomarchy task-30"]
tags: ["protocol", "validation"]
accord:
  status: "ready"
  acceptance: ["Detectable refusals, semantic validation and event/actor/ledger preflight checks occur before any affected record/index/board write; a refused operation leaves those unchanged.", "A temporary regression reproduces a deterministic event/actor/ledger preflight refusal previously occurring after write and asserts unchanged record bytes, relevant ledger contents, index and board refs.", "Late operational write/fsync failures are explicitly outside this preflight-only acceptance: not observed; revisit if seen. No commit-and-compensation framework is added."]
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Herdra approved option (1): predictable refusals/preflight only; no commit-and-compensation for late write/fsync failure.", "Late operational write/fsync failure: not observed; revisit if seen. Do not file a separate Task for that caveat.", "Root Worker is Luna/high, plan first, after task-50. Temporary checks only; no new general transaction/journal framework or push/release."]
  updatedAt: "2026-10-13T13:54:47Z"
createdAt: "2026-10-05T03:52:19Z"
updatedAt: "2026-10-13T13:54:47Z"
blockers: ["task-50"]
effort: "medium"
relatedFiles: ["tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/app/accord.rs", "tandem/src/app/rules.rs", "tandem/src/app/links.rs", "tandem/src/app/review.rs", "tandem/src/app/support.rs", "tandem/src/app/project.rs", "tandem/src/project/events.rs", "tandem/src/project/write.rs"]
---
## Approved narrowed batch C

Algorant/Herdra chose option (1). Put every detectable refusal/preflight check before any affected write; this is the observed issue. The historical actor-id-tracked failure is no longer reproducible because actor IDs moved into Git-local state. Current app mutations still write/archive/delete records before identity/ledger checks during append, exposing deterministic malformed actor/sequence/ledger or unwritable-ledger refusal after the write.

Audit add/update/accord/complete/cancel/rules/link and init/migrate boundaries. Reproduce deterministic existing event preflight failure, then move those checks/required preparation before record changes. Verify affected record, ledger, index and board refs unchanged for refused operations. Work against task-50's finalized event API; don't touch CLI/sync mapping unless a concrete need is escalated.

Scope limitation openly approved: late operational write/fsync failure is not observed; revisit if seen. No guarantee covering arbitrary disk/power-loss or remote/source publication rollback is requested in this Task, and no commit-and-compensation or general transaction/journal framework is authorized. No separate Task for that limitation. Fresh Luna/high Worker with plan before code; temporary checks removed before ready.