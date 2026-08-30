---
id: papercut-2
title: "worker_integrate refuses as cross-owner despite successful worker_recover"
status: "resolved"
createdAt: "2026-08-17T20:54:48Z"
updatedAt: "2026-08-30T12:51:25Z"
references: ["task-230"]
tags: ["herdr", "integration", "ownership", "worker"]
resolution:
  note: "Resolved externally by Dotfiles Pi configuration task-253. Pi Agency now scopes Worker integration outcomes, delivery records, and in-flight merge deduplication by repository identity (`repoKey`) while preserving same-repository cross-owner refusal. Durable task-230 records confirm the original failure was a cross-repository ID collision: a Dotfiles task-230 integration outcome blocked Tandem task-230 despite successful recovery."
  resolvedAt: "2026-08-30T12:51:25Z"
---
Worker `worker-task-230-0def6a42` was started, reworked twice, and reported normally in this session. `worker_integrate` (both `merge` and `inspect`) refused with:

> Durable integration outcomes for task-230 belong to another owner session or workspace. Cross-owner integration is refused.

`worker_status` showed a healthy Worker with correct routing: `integration_target_branch: main`, `integration_target_checkout_path: /home/ivan/Projects/tandem`, `orchestrator_workspace_id: wE`, `integration_target_workspace_id: wE`.

`worker_recover` succeeded and reported reattachment into owner session `01a00de5-2fc8-7b91-8f28-5e818727c161` with a new delivery cycle, but `worker_integrate` continued to refuse with the identical message. Recovery therefore addressed the session half of the ownership check but not the workspace half.

Impact: verified, clean work could not be integrated through the intended tool path, and the orchestrator had to stop and consult the user rather than bypass a refused authority boundary with a manual git merge.

Possible improvement: distinguish the session mismatch from the workspace mismatch in the error text, and state which workspace is expected versus observed, so the operator can tell whether `worker_recover` is even the right remedy.

