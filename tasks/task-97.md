---
id: task-97
uid: 16a91e35-d0b2-467b-8fb8-3230123b11fc
type: task
title: "complete delivers and accepts in one write: tandem complete --summary/--evidence/--file-changed"
state: todo
priority: "low"
effort: "small"
references: [".pi task-566", "herdra task-68"]
accord:
  status: "ready"
  acceptance: ["`tandem complete <id> --summary … --evidence …` on a claimed Task archives it as completed with that summary and evidence in one write and one sync.", "Without those flags complete behaves as today."]
  updatedAt: "2026-11-06T15:04:12Z"
createdAt: "2026-11-06T15:04:12Z"
updatedAt: "2026-11-06T15:04:12Z"
---

## Description


## Description

Requested by .pi task-566 (herdra task-68, Algorant via Herdra m6d76329d): Pi's worker_finish should record an accepted outcome with one Tandem write and one sync. Tandem 0.17.4 has no single deliver+accept write: `tandem complete` takes only --note/--reviewer/--fixed-by, so a claimed Task needs `accord deliver` (write + sync, ~1.4–2.1 s) then `complete` (write + sync, ~1.5–1.7 s).

Pi now works around it by delivering at the Worker's ready report (`accord deliver`), so worker_finish makes only `tandem complete`. A combined write would still remove one write and one sync per Worker hand-off (the report-time delivery), and lets an owner accept claimed work it verified itself in one step.

Proposed: `tandem complete <id> --summary <s> --evidence <e>... [--file-changed <f>...]` on a claimed/rework Task records the delivery fields and accepts/archives atomically, with one sync. Optional, separate question for Algorant: a way to return before the sync finishes (sync off the caller's critical path) where that is safe.

