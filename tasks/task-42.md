---
id: task-42
uid: d1b1095a-2878-48c5-a817-12579d1c9288
type: task
kind: papercut
title: "Offer parent closure after last Subtask and flag finished blockers"
state: todo
priority: "low"
effort: "medium"
tags: ["workflow", "epic", "accord"]
accord:
  status: "blocked"
  acceptance: ["Finishing a Task's last Subtask offers explicit parent-closure confirmation and records the parent outcome once, without automatic closure or fabricated evidence.", "A documented existing command/view flags blockers pointing to already-finished Tasks without silently changing the blocker list."]
  validation: ["$ tandem --version"]
  constraints: ["Low-priority, unstarted backlog; no implementation authorized by this board edit."]
  note: "Parked: Algorant explicitly kept this unstarted; 0.17.1–0.17.4 did not implement either remaining UX item. Re-check when task-97's completion-path change lands, or Algorant explicitly resumes. Do not treat this triage as approval to implement."
  updatedAt: "2026-11-08T03:00:05Z"
createdAt: "2026-09-22T19:50:16Z"
updatedAt: "2026-11-22T16:28:29Z"
---
## Scope trimmed by Algorant via Herdra

Keep low priority and unstarted. Only two items remain:
1. When a Task's last Subtask finishes, offer to close the parent Task, with explicit confirmation and its outcome recorded once. Do not fabricate delivery evidence or silently auto-close.
2. Flag blocker references pointing to already-finished Tasks so people can see stale dependency links; do not silently clear or rewrite them.

Epic-specific closure/exemptions, guided Epic evidence, cross-repo closure tracking, event-collision investigation and mistaken-Epic cleanup are removed from this Task. Define exact finished/eligible predicates against current protocol when work is authorized; no implementation now.

## Staleness triage
Owner decision: parked, retaining Algorant's explicit deferral. Re-checked 2026-11-21: task-97 (complete delivers and accepts in one write) is still todo on the board, so the named trigger has not fired. Current code still archives only the selected Task and no view flags blockers pointing at finished Tasks. No change; keep low priority.

Re-check trigger: task-97's completion-path change lands, or Algorant explicitly resumes these two UX items. External historical context: Pi board task-100 (not a local Tandem record).