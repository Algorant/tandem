---
id: task-49-2
uid: f850dead-a40b-4734-ac08-ac1cebd86ff0
type: task
title: "Step 2: write protocol 0.4.0 and record the decision"
priority: "high"
effort: "small"
parentId: "task-49"
tags: ["protocol", "docs"]
accord:
  status: "accepted"
  acceptance: ["protocol/README.md and protocol/plan/spec.md define uid, provisional handles, numbering at first sync, board and safety-copy storage, sync, merge and conflict semantics, and the removal of checkpoint.", "A new Decision records the approved sync design; decision-7 is amended for the actor-id location and decision-8 is narrowly superseded for storage, migration, and checkpoint removal."]
  claimedAt: "2026-09-30T04:51:40Z"
  deliveredAt: "2026-09-30T04:51:40Z"
  summary: "Protocol 0.4.0 is documented in protocol/README.md (Sync and Migration sections, uid and provisional IDs, actor identity location) and protocol/plan/spec.md; decision-9 records the design; decision-7 carries an amendment; decision-9 narrowly supersedes decision-8."
  evidence: ["protocol/README.md: new Sync and Migration sections, 0.4.0 workspace and identity rules; no checkpoint section remains (grep).", "protocol/plan/spec.md updated to 0.4.0 with uid, provisional IDs, discovery, and the 27-leaf CLI.", "decision-9 created and accepted; decision-7 amended for tandem-actor-id location."]
  filesChanged: ["protocol/README.md", "protocol/plan/spec.md", "protocol/plan/independent-metadata-sync.md"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-09-30T04:51:40Z"
createdAt: "2026-09-30T03:54:17Z"
updatedAt: "2026-09-30T04:51:40Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-30T04:51:40Z"
resolution:
  outcome: "completed"
  reviewer: "pi-orchestrator"
---

