---
id: task-51
uid: eb7996da-f2bf-48ff-b3c3-c24e76495763
type: task
kind: papercut
title: "Workspace docs describe the pre-0.3.0 layout"
state: todo
priority: "low"
relatedFiles: ["docs/workspace/index.md", "protocol/README.md"]
tags: ["docs"]
accord:
  status: "ready"
  acceptance: ["docs/workspace/index.md matches the current protocol layout in protocol/README.md."]
  updatedAt: "2026-09-30T03:41:03Z"
createdAt: "2026-09-30T03:41:03Z"
updatedAt: "2026-09-30T03:41:03Z"
---

## Description

Found during task-48. docs/workspace/index.md still documents board/, papercuts/, and a shared events.jsonl, while protocol 0.3.0 uses tasks/, decisions/, rules/, logs/, and per-actor events/.
