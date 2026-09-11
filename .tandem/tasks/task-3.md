---
id: task-3
type: task
title: "CLI: add decision writes to .tandem/tasks"
state: "in-progress"
priority: "low"
tags: ["papercut"]
accord:
  status: "delivered"
  acceptance: ["decisions/ is the only runtime path for decision documents (D46)"]
  claimedAt: "2026-09-11T14:22:41Z"
  deliveredAt: "2026-09-11T14:23:03Z"
  summary: "Reconciled decision storage fix already shipped in 0.12.1. Algorant approved audit-based closeout."
  evidence: ["Current 0.13.1 native CLI scratch probe created decision-1.md only under .tandem/decisions/ and asserted no same-ID file in .tandem/tasks/. Probe ran without changing project records.", "Code routes creation through project.decisions_dir(); commits79c375c and5860955 are ancestors of tandem-v0.12.1. RELEASES.md records dedicated decision storage.", "The related decision reload defect was separately verified and completed as task-11. Decision editing remains open as task-2 and is not conflated with storage."]
  filesChanged: ["tandem/src/app/decisions.rs", "tandem/src/project/mod.rs"]
  updatedAt: "2026-09-11T14:23:03Z"
createdAt: "2026-08-31T20:53:43Z"
updatedAt: "2026-09-11T14:23:03Z"
assignee: "pi-orchestrator"
---

## Description

Migrating surfaced this: 'tandem add decision' wrote decision-N.md into .tandem/tasks/ instead of .tandem/decisions/. Reader scans both, so nothing visibly broke, but the layout contract is violated and dir-based tooling breaks.
