---
id: task-19
type: task
title: "Remove obsolete machine-local Pi renderer dependency from the project adapter"
state: "in-progress"
priority: "medium"
references: ["task-18"]
tags: ["pi-tandem", "release"]
accord:
  status: "claimed"
  acceptance: ["Project-local pi-tandem loads without importing any deleted machine-specific Pi UI path and uses the SDK default tool renderer.", "Existing adapter argument/CLI smoke tests and the release gate pass against the0.12.4 source binary; installed Pi resources are unchanged."]
  claimedAt: "2026-09-06T00:27:41Z"
  constraints: ["User explicitly approved this bounded adapter fix to unblock release0.12.4.", "Remove the obsolete rendering path rather than copy a renderer, add a fallback import, or modify canonical /home/ivan/.pi resources.", "No broader adapter API or workflow redesign."]
  updatedAt: "2026-09-06T00:27:41Z"
createdAt: "2026-09-06T00:27:35Z"
updatedAt: "2026-09-06T00:27:41Z"
assignee: "pi-orchestrator"
---

