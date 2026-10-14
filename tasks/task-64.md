---
id: task-64
uid: b7218abb-725a-482e-8081-450b4e71173c
type: task
kind: "papercut"
title: "Refused mutations must not leave a written record"
state: "in-progress"
priority: "low"
references: ["Algomarchy task-30"]
tags: ["protocol", "validation"]
accord:
  status: "delivered"
  acceptance: ["Detectable refusals, semantic validation and event/actor/ledger preflight checks occur before any affected record/index/board write; a refused operation leaves those unchanged.", "A temporary regression reproduces a deterministic event/actor/ledger preflight refusal previously occurring after write and asserts unchanged record bytes, relevant ledger contents, index and board refs.", "Late operational write/fsync failures are explicitly outside this preflight-only acceptance: not observed; revisit if seen. No commit-and-compensation framework is added."]
  claimedAt: "2026-10-14T01:54:09Z"
  deliveredAt: "2026-10-14T02:11:20Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Herdra approved option (1): predictable refusals/preflight only; no commit-and-compensation for late write/fsync failure.", "Late operational write/fsync failure: not observed; revisit if seen. Do not file a separate Task for that caveat.", "Root Worker is Luna/high, plan first, after task-50. Temporary checks only; no new general transaction/journal framework or push/release."]
  summary: "Preflight-only prepared ledger guard independently verified at ffab0f6: actor/ledger/name/payload checks precede affected writes/archives; authoritative IDs and hierarchy→ledger lock order retained. Parent native checks and24 refusal/eight-concurrent-add checks pass. No late-fsync/transaction guarantee, no CLI/sync mapping or init/migrate changes. Technical fix ready for parent acceptance."
  evidence: ["Parent reviewed9-file diff264insertions/164deletions. EventSpec validates name/objectdata before mutation; prepared ledger holds actor file lock across mutation+append, reserves contiguous sequences, binds authoritative ID after existing create_record. Single appendpath, no compatibility/framework.", "Parent independently ran just dev-check, fmt, strictclippy anddiffcheck: passed337unit+76integration existingtests and Git-backed nativeworkflow smoke. Existing concurrent append tests exercise prepared path; no new permanent test/fixture.", "Parent disposable Git-backed replica seeded real source Git index plus records/decision/rule/deliveredwork. 24 deterministic refusals =12 native operations × malformedactor and invalidledgerseq: add/updateTask,add/updateDecision,claim,complete,cancel,rulesadd/edit/delete,linkadd,review. Every failed command gave expected actor/event error; exact record/log/rule/decision/ledger bytes/existence, real .git/index, board metadata andallrefs unchanged.", "Parent restored own tempactor/ledger and ran8 concurrentnativeAdds: allsuccess,8 distinctIDs,8 events andcontiguousactorseq; no lockdeadlock. Fixture cleanup in finally, temporaryhelper removed; evidenceJSON /tmp/task64-parent-refusals-evidence.json retained.", "Lock audit: existingHierarchyLocks acquired before prepare; nonhierarchy paths do not subsequentlyacquirehierarchy whileholdingevent lock. Destination+snapshot archive preflight before actorprep; currentfile snapshot checks preserved aroundblockinglock acquisition.", "No docs/CLI/sync final-IDmapping,init/migrate,protocolversion or source releasechanges. Late operationalwrite/fsync remains explicitly notobserved/revisitifseen outsidecriteria; no newcompensation/journal.", "Context1 authorizes parent acceptance of approved technical work after verification. worker_finish integrates and owns Task acceptance/presentation/checkout cleanup. No push/release/deploy; task63+77 follows finalizedAPI."]
  filesChanged: ["tandem/src/project/events.rs", "tandem/src/project/write.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/app/accord.rs", "tandem/src/app/rules.rs", "tandem/src/app/links.rs", "tandem/src/app/review.rs"]
  updatedAt: "2026-10-14T02:11:20Z"
createdAt: "2026-10-05T03:52:19Z"
updatedAt: "2026-10-14T02:11:20Z"
blockers: ["task-50"]
effort: "medium"
relatedFiles: ["tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/app/accord.rs", "tandem/src/app/rules.rs", "tandem/src/app/links.rs", "tandem/src/app/review.rs", "tandem/src/app/support.rs", "tandem/src/app/project.rs", "tandem/src/project/events.rs", "tandem/src/project/write.rs"]
assignee: "worker-task-64-0f2a6759"
---
## Approved narrowed batch C

Algorant/Herdra chose option (1). Put every detectable refusal/preflight check before any affected write; this is the observed issue. The historical actor-id-tracked failure is no longer reproducible because actor IDs moved into Git-local state. Current app mutations still write/archive/delete records before identity/ledger checks during append, exposing deterministic malformed actor/sequence/ledger or unwritable-ledger refusal after the write.

Audit add/update/accord/complete/cancel/rules/link and init/migrate boundaries. Reproduce deterministic existing event preflight failure, then move those checks/required preparation before record changes. Verify affected record, ledger, index and board refs unchanged for refused operations. Work against task-50's finalized event API; don't touch CLI/sync mapping unless a concrete need is escalated.

Scope limitation openly approved: late operational write/fsync failure is not observed; revisit if seen. No guarantee covering arbitrary disk/power-loss or remote/source publication rollback is requested in this Task, and no commit-and-compensation or general transaction/journal framework is authorized. No separate Task for that limitation. Fresh Luna/high Worker with plan before code; temporary checks removed before ready.

## Concrete plan approved

Read-only /tmp/task64-preflight-plan.md reproduces malformed Git-local actor UUID refusal after native add created task-2, while ledger/pending ref stayed unchanged. Approved narrow actor-ledger preparation guard holding the existing ledger lock through record mutation+append, consistent HierarchyLock→event lock ordering, contiguous sequences for operations with multiple events. IDs for new records still come from existing create_record authority; preparation must not invent/preallocate a separate ID path (bind returned ID for append). Semantic checks/payload validation before preparation; first record/archive/delete only after actor/ledger checks. Include review; init/migrate untouched absent concrete failure. No generic transaction/journal or late-fsync guarantee. Temporary exact record/ledger/index/board-ref refusal and concurrency checks; native suite/fmt/strictclippy. Parent explicitly approves same retained Worker; no second-plan gate.