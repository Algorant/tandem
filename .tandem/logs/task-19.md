---
id: task-19
type: task
title: "Remove obsolete machine-local Pi renderer dependency from the project adapter"
priority: "medium"
references: ["task-18"]
tags: ["pi-tandem", "release"]
accord:
  status: "accepted"
  acceptance: ["Project-local pi-tandem loads without importing any deleted machine-specific Pi UI path and uses the SDK default tool renderer.", "Existing adapter argument/CLI smoke tests and the release gate pass against the0.12.4 source binary; installed Pi resources are unchanged."]
  claimedAt: "2026-09-06T00:27:41Z"
  deliveredAt: "2026-09-06T00:35:49Z"
  constraints: ["User explicitly approved this bounded adapter fix to unblock release0.12.4.", "Remove the obsolete rendering path rather than copy a renderer, add a fallback import, or modify canonical /home/ivan/.pi resources.", "No broader adapter API or workflow redesign."]
  summary: "Removed obsolete absolute Pi UI renderer import and all custom renderer registrations; local adapter now uses SDK default rendering. Shipped in0.12.4 commit1af1db8."
  evidence: ["All nine tool definitions register without machine-local UI imports; smoke asserts renderCall/renderResult/renderShell are undefined.", "Existing project-local smoke, relationship-smoke and pi-runtime-smoke scripts pass against source-built0.12.4.", "Full just release0.12.4 gate passed and GitHub Release run34001441498 succeeded. No canonical /home/ivan/.pi resources were edited."]
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/README.md", "RELEASES.md"]
  updatedAt: "2026-09-06T00:35:58Z"
createdAt: "2026-09-06T00:27:35Z"
updatedAt: "2026-09-06T00:35:58Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-06T00:35:58Z"
resolution:
  outcome: "completed"
---

