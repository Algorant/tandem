---
id: papercut-3
title: "worker_integrate refuses task-232 as cross-owner despite matching orchestrator workspace"
status: open
createdAt: "2026-08-22T15:27:26Z"
updatedAt: "2026-08-22T15:27:26Z"
references: ["task-232"]
tags: ["integration", "pi-agency", "worker"]
---
`worker_integrate` with action=inspect for task-232 fails with "Durable integration outcomes for task-232 belong to another owner session or workspace. Cross-owner integration is refused." Repeated twice with the same result.

The Worker was started from this session and `worker_status` for worker-task-232-75eae6d3 reports tokens that match the current orchestrator: `orchestrator_workspace_id: w3P`, `integration_target_workspace_id: w3P`, `integration_target_branch: main`, `integration_target_checkout_path: /home/ivan/Projects/tandem`, `role: worker`, `task_id: task-232`.

The ownership check appears to be keyed on something other than the workspace id recorded in those tokens, most likely an owner session identity that did not survive a rework cycle or an orchestrator reload. Read-only inspect is refused as well, not just merge, so there is no way to review the captured target through the tool.

Impact: blocks the documented worker_integrate path and forces either a manual git merge outside Worktrunk or abandoning the Worker checkout.
