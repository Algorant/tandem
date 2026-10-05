---
id: task-64
uid: b7218abb-725a-482e-8081-450b4e71173c
type: task
kind: "papercut"
title: "Refused mutations must not leave a written record"
state: todo
priority: "low"
references: ["Algomarchy task-30"]
tags: ["papercut"]
accord:
  status: "ready"
  acceptance: ["Every tandem mutation that exits non-zero leaves the workspace unchanged: no new or modified record, index entry or board commit.", "A regression covers at least one refusal that previously fired after the write (a preflight or validation error during add)."]
  updatedAt: "2026-10-05T03:52:19Z"
createdAt: "2026-10-05T03:52:19Z"
updatedAt: "2026-10-05T03:52:19Z"
---

## Description

## Description

Reported from the Algomarchy workspace (its task-30), 2026-09-28 on tandem 0.13.7: `tandem add decision` exited 1 with "actor-id is tracked by Git", yet it had already written `.tandem/decisions/decision-1.md`. A failure exit after a successful write invites duplicate records when the caller retries.

That specific refusal can no longer happen: decision-9 moved the actor identity to `<git-dir>/tandem-actor-id` outside Git (confirmed on 0.16.2 in Algomarchy, where nothing under `.tandem/` tracks an actor ID). The general hazard remains unverified: any mutation whose validation or preflight runs after the record is written can report failure while leaving the record behind.

## Approach

Audit add/update/close/accord paths for checks that run after the write; move them before the write, or roll back the write on failure. Add a regression for one representative refusal.
