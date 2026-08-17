---
id: papercut-2
title: "worker_integrate refuses as cross-owner despite successful worker_recover"
status: open
createdAt: "2026-08-17T20:54:48Z"
updatedAt: "2026-08-17T20:54:48Z"
references: ["task-230"]
tags: ["herdr", "integration", "ownership", "worker"]
---
Worker `worker-task-230-0def6a42` was started, reworked twice, and reported normally in this session. `worker_integrate` (both `merge` and `inspect`) refused with:

> Durable integration outcomes for task-230 belong to another owner session or workspace. Cross-owner integration is refused.

`worker_status` showed a healthy Worker with correct routing: `integration_target_branch: main`, `integration_target_checkout_path: /home/ivan/Projects/tandem`, `orchestrator_workspace_id: wE`, `integration_target_workspace_id: wE`.

`worker_recover` succeeded and reported reattachment into owner session `01a00de5-2fc8-7b91-8f28-5e818727c161` with a new delivery cycle, but `worker_integrate` continued to refuse with the identical message. Recovery therefore addressed the session half of the ownership check but not the workspace half.

Impact: verified, clean work could not be integrated through the intended tool path, and the orchestrator had to stop and consult the user rather than bypass a refused authority boundary with a manual git merge.

Possible improvement: distinguish the session mismatch from the workspace mismatch in the error text, and state which workspace is expected versus observed, so the operator can tell whether `worker_recover` is even the right remedy.

