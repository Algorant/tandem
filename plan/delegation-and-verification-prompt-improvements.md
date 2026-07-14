# Delegation and verification prompt improvements

## Why this was discussed

During verification of `task-125`, the worker produced a focused commit, a clean worktree, exact validation evidence, changed-file and checkout details, risks/blockers, and a final statement that it was ready for parent delivery. The task still had a claimed accord, so the parent incorrectly classified the handoff as **NOT READY** and sent the worker back to perform Tandem delivery.

That exposed an ambiguity between two different concepts:

1. **Worker handoff readiness**: implementation and evidence are ready for parent review.
2. **Tandem delivery state**: the orchestrator records that handoff through `shep_deliver` and moves the task to Validation.

The established Shep model assigns Tandem delivery, acceptance, completion, integration, push, and cleanup to the parent/orchestrator. A worker that ends with “ready for parent delivery” has therefore followed the intended workflow; a claimed accord alone does not make its handoff incomplete.

## Findings

- The reusable `/delegate` prompt already says that the parent/orchestrator owns Tandem delivery and later lifecycle actions.
- Pi-shep's generated worker prompt repeats the same ownership rule and requests an integration-ready handoff.
- The phrase “delivered integration-ready work” in `/verify` is ambiguous because “delivered” can mean either reporting a handoff or mutating the Tandem accord.
- `/verify` does not explicitly tell the parent to call `shep_deliver` after confirming a complete handoff.
- Plain requests such as “delegate task-125” should be sufficient: the orchestrator should use `shep_delegate` and rely on pi-shep's standardized worker prompt rather than improvising lifecycle instructions.
- Separately, the installed `/usr/bin/tandem 0.4.3` omitted `parentId` from the task show response even though the raw task document contained it. That can hide relationship context from Shep, but it is not a worker-handoff failure.

## Recommended `/delegate` clarification

Keep the existing handoff checklist and add an explicit terminal contract:

```text
The worker must not call `shep_deliver` or `tandem accord deliver`.
End the handoff with exactly `READY FOR PARENT DELIVERY` or
`NOT READY: <reason>`. The parent owns all lifecycle transitions.
```

This makes “delivery” an orchestrator action and prevents a final prose handoff from being mistaken for an accord transition.

## Recommended `/verify` clarification

Verification should explicitly separate artifact readiness from lifecycle state:

```text
Treat worker handoff readiness and Tandem lifecycle state as separate.

A worker handoff is READY when it includes:
- a focused commit or explicit no-commit reason
- branch/worktree details
- intended and unexpected changed files
- exact validation commands and results
- clean/reported `git status --short`
- risks/blockers
- `READY FOR PARENT DELIVERY`

If evidence is missing, send precise feedback with `shep_send` and stop
as NOT READY.

If the handoff is ready but the accord is still claimed, the parent calls
`shep_deliver`; do not send the worker back merely to perform the delivery
transition. Then inspect the commit/diff and independently validate it.
```

## Intended reusable workflow

1. Delegate an existing Tandem task with `shep_delegate`.
2. The worker implements only that task, validates it, creates one focused commit, and reports the standardized handoff.
3. The parent uses `shep_check` and inspects the commit, diff, status, checkout, evidence, and risks.
4. Missing implementation evidence produces **NOT READY** feedback through `shep_send`.
5. A complete handoff is recorded by the parent with `shep_deliver` when not already delivered.
6. The parent performs independent review and validation.
7. Objective, non-visual work may be integrated, accepted, completed, logged, and cleaned up automatically when it passes.
8. Visual, UX, product, manual, high-risk, or ambiguous work remains in Validation for human judgment.

The key rule is: **claimed versus delivered is lifecycle state; READY versus NOT READY is evidence quality. Do not conflate them.**
