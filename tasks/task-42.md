---
id: task-42
uid: d1b1095a-2878-48c5-a817-12579d1c9288
type: task
kind: papercut
title: "Offer parent closure after last Subtask and flag finished blockers"
state: todo
priority: "low"
effort: "medium"
references: ["~/.pi task-100"]
tags: ["workflow", "epic", "accord"]
accord:
  status: "ready"
  acceptance: ["Finishing a Task's last Subtask offers explicit parent-closure confirmation and records the parent outcome once, without automatic closure or fabricated evidence.", "A documented existing command/view flags blockers pointing to already-finished Tasks without silently changing the blocker list."]
  validation: ["$ tandem --version"]
  constraints: ["Low-priority, unstarted backlog; no implementation authorized by this board edit."]
  updatedAt: "2026-10-13T13:27:40Z"
createdAt: "2026-09-22T19:50:16Z"
updatedAt: "2026-10-13T13:27:40Z"
---
## Scope trimmed by Algorant via Herdra

Keep low priority and unstarted. Only two items remain:
1. When a Task's last Subtask finishes, offer to close the parent Task, with explicit confirmation and its outcome recorded once. Do not fabricate delivery evidence or silently auto-close.
2. Flag blocker references pointing to already-finished Tasks so people can see stale dependency links; do not silently clear or rewrite them.

Epic-specific closure/exemptions, guided Epic evidence, cross-repo closure tracking, event-collision investigation and mistaken-Epic cleanup are removed from this Task. Define exact finished/eligible predicates against current protocol when work is authorized; no implementation now.