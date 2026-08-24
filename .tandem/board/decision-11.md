---
id: decision-11
type: decision
title: "Revive review.status as the judgment signal and stop accord from driving workflow state"
status: "accepted"
date: "2026-08-24"
deciders: ["ivan", "pi"]
context: "`validation` became the default parking state for delivered work. Three tasks in the 2026-08-22 session (task-237, task-238, task-240) were objective, verifiable work that was nonetheless routed through `validation` and then required a separate remembered `complete` step.\n\nInvestigation found three parallel tracks, one of which was entirely unused:\n\n| Track | Field | Used |\n| --- | --- | --- |\n| Workflow | `state` | yes |\n| Agreement | `accord.status` | yes |\n| Judgment | `review.status` | never |\n\nThe protocol spec already defines \"Request review\" and \"Accept review / request changes\" operations that set and resolve `review.status`, and states that review does not automatically move `state`. The intended design was that review is the signal and `validation` is a column that follows from it. Practice inverted this: the column was used and the signal never set, so `review.status=missing` warned on every completion and carried no information.\n\nThe structural defect was in accord-to-state coupling. The spec had both `delivered` and `accepted` move work into `validation`, so acceptance, which is the judgment itself, left work parked in the waiting room."
consequences: ["No accord action moves workflow state except `claim` moving `todo` to `in-progress`. `deliver`, `accept`, `rework`, `block`, and `fail` leave `state` untouched.", "`review.status: pending` is the only way a task enters `validation`. Manual `tandem move <id> validation` without a pending review is rejected (E7).", "Objective delegated work never enters `validation`: the orchestrator verifies, records `deliver`, `accept`, then `complete`. Unset `review.status` is the normal, correct case.", "The completion warning inverts. `review.status=missing` becomes silent. Completing with `review.status: pending` becomes a hard error, since it discards a request that was explicitly made (E5).", "Review resolution is human-only. An orchestrator may request a review but may not resolve one (E2).", "Review-reject and `accord rework` stay separate actions; a rejected review implies accord rework (E4).", "`accord rework` must clear a pending review, or the task returns to `in-progress` carrying a stale flag and cannot complete (E3).", "Review is requestable independently of accord status, so direct orchestrator work with no Worker can still request human judgment (E6).", "Review is requestable only at the delegated Task boundary, not on Subtasks. Epics are exempt and complete when their Tasks do (E8).", "BLOCKING AND UNRESOLVED: delivered-but-untriaged work now sits in `in-progress` with `accord: delivered`, visually identical to work still in flight. The Board must surface it before this ships, or a visible parking lot is replaced by an invisible one (E1). This is a visual TUI change requiring human terminal validation and is not delegable to autonomous acceptance."]
alternatives: ["O1 decouple entirely: accord never touches state and the orchestrator moves everything explicitly. Rejected as the stated option because it strands work in `in-progress`, though the accepted design converges on it for every action except `claim`.", "O3 accepted implies completion: `accept` archives directly to logs. Rejected because it merges two genuinely different operations and removes the moment where task-228 tidy-up is meant to run, while making acceptance hard to reverse.", "R2 remove the review track: delete `review.status` in v0. Rejected because it deletes the only field designed to record why work is waiting, solving warning noise by removing the feature that addresses the underlying problem.", "R3 leave review dormant and stop warning: zero implementation, removes noise, decides nothing. Rejected as deferral."]
references: ["task-239", "task-239-4", "task-228", "papercut-4", "papercut-5"]
tags: ["protocol", "workflow", "accord", "review", "validation"]
createdAt: "2026-08-24T23:03:18Z"
updatedAt: "2026-08-24T23:03:18Z"
---

## Status

Accepted 2026-08-24, with E1 unresolved and blocking implementation.

## Decision

Revive `review.status` as the judgment signal (R1) and stop accord status from driving workflow state (O2, extended).

Combining R1 and O2 forced a clarification recorded here rather than left to implementation: if `review.status: pending` is what places work in `validation`, then `deliver` cannot also place it there, or the flag is decorative. The accepted rule is therefore that **no accord action moves workflow state except `claim`**. This is closer to the rejected O1 than to O2 as originally framed, and it inherits O1's stranding risk, which is exactly what E1 addresses.

## Resulting flow

```
todo --claim--> in-progress --deliver--> in-progress (accord: delivered)
                                              |
                                    orchestrator triages
                                     /                  \
                    objective, verified            needs human
                            |                            |
                     accept, complete            request review
                            |                            |
                          logs                  validation (review: pending)
                                                    /            \
                                            human accepts    human rejects
                                                  |                |
                                           accept, complete   in-progress
```

## Scenarios

**Objective delegated work.** Orchestrator creates the Task and starts a Worker; `claim` moves it to `in-progress`. The Worker implements, reports, and runs no `tandem` commands per papercut-5. The orchestrator verifies independently, then records `deliver`, `accept`, `complete`. The task never enters `validation` and `review.status` stays unset.

**Delegated work needing judgment.** As above through verification, then the orchestrator records `deliver` and requests review with a reason. The task moves to `validation` with `review.status: pending`. The human accepts or requests changes.

**Rejected work.** Review resolves as rejected, the task returns to `in-progress`, and the orchestrator issues `worker_rework` or fixes it directly.

## Execution

task-239-1, task-239-2, task-239-3, and task-228 execute this decision and must not restate or reinvent it. E1 must be resolved before task-239-2 lands.
