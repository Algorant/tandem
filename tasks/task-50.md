---
id: task-50
uid: ef34cbd8-c699-4582-a2af-2ae1ed59bd51
type: task
title: "Event envelope in code contradicts protocol spec (summary vs data)"
state: todo
priority: "low"
relatedFiles: ["protocol/plan/spec.md", "tandem/src/protocol/event.rs", "tandem/src/project/events.rs"]
tags: ["papercut", "protocol"]
accord:
  status: "ready"
  acceptance: ["The protocol spec and the executable event envelope agree on whether `summary` and `data` are required, with a test asserting the chosen contract."]
  updatedAt: "2026-09-30T03:41:03Z"
createdAt: "2026-09-30T03:41:03Z"
updatedAt: "2026-09-30T03:41:03Z"
---

## Description

Found during task-48. protocol/plan/spec.md requires event-specific `data` and forbids a required free-text summary, but tandem/src/protocol/event.rs and project/events.rs require `summary` and make `data` optional. Readers of the spec get the wrong contract.
