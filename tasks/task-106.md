---
id: task-106
uid: e9bd246f-7875-443a-9c81-55380937015a
type: task
kind: "papercut"
title: "Decision add can stall and duplicate across an update, and a mistaken Decision cannot be discarded"
state: todo
priority: "low"
tags: ["cli", "sync"]
accord:
  status: "ready"
  acceptance: ["A `tandem add decision` interrupted by a mid-write stall (for example an update replacing the running binary) either completes or fails cleanly without leaving a partly written record that a retry duplicates.", "A Decision added by mistake can be discarded through a documented command, with the same safety as Task cancellation (no silent history rewrite) instead of only supersede/retitle.", "Both fixes are verified end-to-end on a disposable Git-backed board, including a retry after a stalled write."]
  updatedAt: "2026-11-11T04:35:03Z"
createdAt: "2026-11-11T04:35:03Z"
updatedAt: "2026-11-11T04:35:03Z"
---

## Description

Reported by Herdra m8461b3b6 from the homelab board (outside this repository's board).

Observed on the homelab board while Tandem was being updated to 0.17.6: `tandem add decision` stalled mid-write. Retrying produced further probe decisions, leaving duplicates as decision-6 through decision-10. Decisions have no cancel/discard action, so the owner retitled them "duplicate/probe — ignore" and marked them superseded. That is the workaround, not a remedy: the records still occupy IDs and the board.

Two separate defects to cover:
1. A create that stalls across a binary update (or any interruption) must not leave a record that a retry then duplicates. Either the write completes and the retried command reports the existing record, or it fails with nothing written; a half-written Decision file must never become a second record.
2. There is no way to cancel or discard a Decision added by mistake. `tandem cancel` is Task-only and Decisions have no Accord, so an erroneous Decision can only be retitled/superseded. Decide the smallest safe discard path (for example a Decision-only cancel that archives to Logs with a reason, consistent with Task cancellation rather than deletion).

No change now; capture only. Any fix must define what happens to an interrupted write on the local board versus a synced board, and must not rewrite archived history.
