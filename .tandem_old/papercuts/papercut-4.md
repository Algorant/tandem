---
id: papercut-4
title: "Worker integration completes without verifying Tandem board state"
status: "resolved"
createdAt: "2026-08-22T23:02:16Z"
updatedAt: "2026-08-30T12:51:35Z"
references: ["task-228", "task-238", "task-239", "decision-11"]
tags: ["accord", "integration", "workers", "workflow"]
resolution:
  note: "Resolved as superseded by decision-11 and the current orchestration contract. Worker integration is intentionally a technical source operation, not a Tandem lifecycle transition. The orchestrator reads and verifies the Task from the main checkout, then owns Accord delivery/acceptance, optional review, completion, and user-facing workflow reporting. Requiring `accord: delivered` before technical integration would conflict with the accepted verification flow."
  resolvedAt: "2026-08-30T12:51:35Z"
---
## Observed

During task-238, the Worker delivered a report and committed clean work, but never ran `tandem accord deliver`. The orchestrator merged the branch into `main` via `worker_integrate` and confirmed disposition via `worker_cleanup`, both of which succeeded. The task remained `state: in-progress`, `accord: claimed`, assigned to a Worker that no longer existed.

The orchestrator then reported to the user that the task was "in validation" — a state it had never queried and that was never true.

## Two defects

1. **No verification gate.** Neither `worker_integrate` nor `worker_cleanup` reads or reports task state. Code can land on `main` while the board still shows the task claimed and in progress, with no signal that the two diverged.

2. **Asserted state without reading it.** The orchestrator described workflow state from assumption rather than from `tandem show`. A misapplied policy is recoverable; a fabricated status is not, because it is indistinguishable from a real one in the conversation record.

## Contributing factor

Worker handoff reports carry `STATUS: ready for parent review` as free text. That reads as a workflow state and invites the reader to treat it as one. It has no relationship to `state` or `accord.status`.

## Candidate remedies

- Require reading the task before integration, and refuse or loudly warn when accord is not `delivered`.
- Have the orchestrator run `tandem accord deliver` on the Worker's behalf when the Worker did not, before integrating.
- Never state workflow state in a user-facing summary without a preceding read.

## Relationship to existing tasks

Distinct from `task-239`, which addresses `validation` being overused as a parking state. This is about integration and board state silently disagreeing, regardless of which state is correct. Also distinct from `task-228`, which covers residue cleanup after the acceptance boundary rather than verification during it.
