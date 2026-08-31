---
id: papercut-8
title: "worker_rework refuses while a Worker is actively working"
status: open
createdAt: "2026-08-25T01:04:38Z"
updatedAt: "2026-08-25T01:04:38Z"
references: ["decision-11", "papercut-2", "papercut-3", "task-239-2"]
tags: ["orchestration", "tooling", "workers", "workflow"]
---
## Observed

`worker_rework` returned `Refusing rework while Worker worker-task-239-2-36f9ddcd is working.`

The orchestrator had just discovered that an instruction it gave minutes earlier was wrong: decision-11 amendment A4 was rewritten to drop a required `--reviewer` flag and a human-only check the Worker had been told to build. The correction was ready before the Worker could plausibly have reached that part of the work.

The tool refused to deliver it. The only options were to cancel the Worker and lose everything in flight, or let it build known-wrong code and pay for a full rework cycle afterward.

## Why the restriction is wrong

Corrections are most valuable while work is in progress. A running Worker is exactly when new information arrives: the orchestrator is reviewing adjacent work, the human is giving feedback, and upstream decisions are still moving. Forcing that information to wait until the Worker has finished guarantees wasted work whenever the orchestrator learns something mid-flight.

The refusal also inverts the normal relationship. A human can interrupt an agent at any time. An orchestrator cannot interrupt the Worker it started, despite owning the task and the instructions.

## Cost in this case

The Worker is building a required `--reviewer` flag and a human-only validation path that will be deleted on arrival. Small in isolation, but the pattern scales with task size, and this is the largest task in the current campaign.

## Candidate remedies

- Allow `worker_rework` while working, delivered as a queued message the Worker reads at its next turn boundary rather than as an interrupt.
- Provide a separate lighter channel for mid-flight corrections that does not open a new handoff cycle.
- If the refusal exists to protect handoff-cycle bookkeeping, decouple message delivery from the handoff cycle rather than blocking the message.

## Not a remedy

Cancelling and restarting the Worker. It discards completed work and re-pays startup cost to deliver one paragraph of correction.
