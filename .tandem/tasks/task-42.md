---
id: task-42
type: task
title: "Reduce Epic/parent closure friction: derived closure, stale-blocker detection, guided evidence"
state: todo
priority: "low"
effort: "medium"
references: ["~/.pi task-100"]
tags: ["workflow", "epic", "accord", "papercut"]
accord:
  status: "ready"
  acceptance: ["Each friction class is confirmed or rejected against current Tandem behavior with a reproduction or source citation", "An approved design addresses Epic/parent closure without silently fabricating delivery evidence", "Stale blockers referencing archived records are detectable by a documented command or output", "The events-file ID collision observation is triaged (bug, expected, or not reproducible)"]
  validation: ["$ tandem --version"]
  updatedAt: "2026-09-22T19:50:16Z"
createdAt: "2026-09-22T19:50:16Z"
updatedAt: "2026-09-22T19:50:16Z"
---

## Description

Filed from the ~/.pi workspace review (~/.pi task-100, 2026-09-22, Tandem 0.13.6). Evidence is from ~/.pi/.tandem logs/events; record IDs below are ~/.pi IDs.

## Observed friction
1. **Epics complete with Accord still `ready`.** 3 of 6 completed Epics (task-85, task-102, task-110) were archived `completed` with accord.status `ready`, no claimedAt/deliveredAt, no delivery summary/evidence; task-85/102 have no reviewer. Events for task-85/102 show only task.created and task.completed. `close complete` warns that completion normally follows a delivered Accord; operators archive anyway (~/.pi task-188 log documents this as expected). Sampled non-Epic parents with subtasks (task-92, 143, 165, 198, 199, 229 ...) all went claim→deliver→accepted, so the gap is Epic-shaped: an Epic is a coordination shell no Worker claims.
2. **Parent acceptance duplicates child completion.** Parents often state "subtask X is accepted" / "every child Task is accepted" (task-92, task-102, task-165, task-198), so the parent is substantively done when children finish yet still requires a separate lifecycle.
3. **Stale blockers.** Blockers referencing already-archived Tasks persist: live example task-158→task-157 and task-233→task-231 had to be cleared by hand (2026-09-22). Archived records retain stale blockers (task-106, 108, 117, 143-4, 147-5, 182-5, 186, 199-4).
4. **Cross-repo closure evidence** (task-37, task-102, task-198, task-209) is asserted in prose only; no structured link to the external workspace record.
5. **Mistaken Epic creation** churn (task-138 Epic + 4 children canceled, recreated as task-143).

## Suggested fixes (evaluate, do not assume all)
- Derived Epic closure: when all children are archived, surface the Epic as ready-to-close and offer one step that records a delivered/accepted Accord summarizing child outcomes (or exempt `kind: epic` from the delivered-Accord warning with an explicit reason).
- Stale-blocker detection: flag (or clear on explicit command) blockers pointing at archived records, e.g. in `list`/`show` output or a `tandem doctor` check.
- Guided closure evidence: prompt for summary/reviewer when completing a parent/Epic without a delivery.
- Optional structured external-evidence references for cross-workspace closure.

Also observed (may be a separate bug): ~/.pi/.tandem/events holds three jsonl files with apparently colliding task IDs (task-98/100/102 appear in two files with different titles), and events for task-110..120 were not found in any file.
