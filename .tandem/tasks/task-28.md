---
id: task-28
type: task
title: "Epic closure: completing a parent epic with delivered children still warns 'complete normally follows a delivered Accord'"
state: todo
priority: "low"
tags: ["papercut", "workflow"]
accord:
  status: "ready"
  acceptance: ["Completing a parent epic whose children are all delivered/accepted does not emit the 'complete normally follows a delivered Accord' warning, OR the warning is understood and suppressed/document handled for epics with all children accepted."]
  updatedAt: "2026-09-09T21:16:54Z"
createdAt: "2026-09-09T21:16:54Z"
updatedAt: "2026-09-09T21:16:54Z"
---

## Description

Source: pi epic task-102 'Executable Worker contracts and disposable attempts' (Algorant). All six child tasks (103-108) are accepted/archived, yet completing the parent epic emitted: 'task-102 has accord.status=ready; complete normally follows a delivered Accord.' This is the same epic-closure friction documented in the pi workspace (task-85 and its children) and in ~/.pi/.tandem. Parent epics whose work is fully delivered by children still sit accord.status=ready, and completion warns. Assess whether parent epics should support a derived/delivered status or a guided closure path (consistent with pi task-100 field notes).
