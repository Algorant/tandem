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
updatedAt: "2026-08-24T23:26:20Z"
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

## Amendments

Two gaps in the original record, found while task-239-1 implemented it in `protocol/plan/spec.md`. Both are corrections to this decision, not deviations from it.

### A1 (2026-08-24) — E3 was incomplete

E3 required `accord rework` to clear a stale pending review, but did not say it must also move the task.

Clearing the review alone leaves the task in `validation` with no pending review: parked with nobody waiting on it, which is exactly the failure this decision eliminates. The Worker implemented E3 as written, so the defect was in this record.

**Corrected rule:** `accord rework` on a task in `validation` clears the pending review **and** returns it to `in-progress`. This is the only accord action besides `claim` that moves workflow state.

`block` and `fail` remain state-neutral and preserve a pending review, because such a task legitimately still awaits its human.

### A2 (2026-08-24) — no CLI surface for review

This decision makes requesting and resolving review the central human-judgment path, but specified no CLI surface, and none exists. `tandem accord` has `claim|deliver|accept|rework|block|fail`; review has nothing.

**Adopted:** `tandem review request|accept|reject <id>`, symmetric with `tandem accord`.

Consequences:

- This adds a command family to the locked v0 CLI list in `AGENTS.md`, which must be updated as part of task-239-2.
- `tandem move <id> --state validation` remains a valid command. It is rejected at runtime when there is no pending review, rather than being removed.
- Review resolution stays human-only per E2, so `accept` and `reject` are operator commands and are not available to Workers or orchestrators acting on their own requests.
### A3 (2026-08-24) — `changes-requested` had no command

A2 specified `tandem review request|accept|reject`, but the protocol already documents three resolve outcomes: `accepted`, `changes-requested`, and `rejected`, with a matching `review.changes_requested` event. A2 left `changes-requested` reachable by no command.

**Adopted:** `tandem review request|accept|changes|reject <id>`.

`changes` and `reject` both resolve a pending review and return the task to `in-progress`. They differ in meaning, not mechanics: `changes` means iterate on this work, `reject` means the work is not acceptable as an approach. `accord fail` remains the signal for abandoning the effort entirely.

Adding a subcommand is preferred over deleting the `changes-requested` status, because the status, its event, and its validation entry are already specified and removing them is the larger change.

### Pattern note

A1, A2, and A3 were each found by a Worker implementing this decision faithfully. In every case the Worker was correct and this record was incomplete. Decisions that introduce a workflow concept should enumerate its full status vocabulary, its events, and its command surface together before delegation, rather than leaving them to be discovered.