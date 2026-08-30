---
id: papercut-5
title: "Worker-side accord transitions do not survive integration"
status: "resolved"
createdAt: "2026-08-22T23:08:02Z"
updatedAt: "2026-08-30T12:51:40Z"
references: ["papercut-4", "task-237", "task-238", "task-239", "decision-11", "task-228"]
tags: ["accord", "integration", "workers", "workflow", "worktrees"]
resolution:
  note: "Resolved by the current Worker and orchestrator contract. Worker kickoff instructions and Pi Agency guidance now prohibit Workers from running Tandem lifecycle transitions; Workers report implementation evidence only. The orchestrator records transitions from the main checkout, and Worker integration requires a clean source, preventing uncommitted worktree-local `.tandem` transitions from being silently integrated or discarded as lifecycle truth."
  resolvedAt: "2026-08-30T12:51:40Z"
---
## Observed

Mechanism behind `papercut-4`, confirmed on task-237.

When instructed to run `tandem accord deliver`, the Worker ran it inside its own worktree. That wrote `.tandem/board/task-237.md` and created `.tandem/events/<actor>.jsonl` there, both uncommitted. The Worker then reported `STATUS: Delivered; task is in validation`, which was true of its worktree and false of the main checkout, which still read `state: in-progress`, `accord: claimed`.

Integration merges committed source changes. Uncommitted board edits stay behind and are destroyed with the worktree at cleanup. The Worker's report was accurate about its own filesystem and misleading about the project.

## Why committing them would also be wrong

Committing `.tandem/` changes from a Worker branch is not the fix. Each worktree has its own ignored `actor-id`, so the Worker allocates event sequence numbers against a ledger the main checkout cannot see. Merging those introduces exactly the cross-checkout `seq` collision that task-238 was filed to prevent.

## Working approach

The orchestrator records Accord transitions in the main checkout after verifying the deliverable, using the Worker's report as evidence. Workers report; they do not transition. This matches pi-agency, which already assigns Tandem transitions to the orchestrator, but neither the skill nor the Worker prompt states that Workers must not run `tandem accord` themselves.

Applied to task-237 and task-238: orchestrator ran deliver, accept, then complete from the main checkout, with the Worker's uncommitted board edits discarded before merge.

## Candidate remedies

- State in pi-agency that Workers must not run `tandem accord` or `tandem complete`; the orchestrator owns all transitions in the main checkout.
- Stop instructing Workers to record Accord transitions in Worker startup prompts.
- Have `worker_integrate` refuse or warn when the Worker source contains uncommitted `.tandem/` changes, rather than silently leaving them behind.
