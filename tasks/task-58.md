---
id: task-58
uid: 498cd99a-cf8d-4c9d-a724-29f01431d70c
type: task
title: "Update or retire extensions/pi-tandem for research and papercut kinds"
state: "in-progress"
priority: "low"
effort: "small"
blockers: ["task-55"]
references: ["decision-10", "task-55"]
tags: ["pi-tandem", "taxonomy"]
accord:
  status: "claimed"
  acceptance: ["Either `extensions/pi-tandem` classifies and lists papercuts by `--kind papercut` (no tag path remains), or the adapter is retired with that decision recorded.", "The adapter's tests or smoke checks reflect the chosen outcome."]
  claimedAt: "2026-10-01T23:36:03Z"
  updatedAt: "2026-10-01T23:36:03Z"
createdAt: "2026-10-01T22:21:05Z"
updatedAt: "2026-10-01T23:36:03Z"
relatedFiles: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests", "docs/extensions/index.md"]
assignee: "worker-task-58-da6de440"
---

## Description

Explicit adapter Task, as rule never-2 requires. `tandem_papercut` currently adds `--tag papercut`, and `list` filters by that tag. The ~/.pi configuration uses its own Tandem tools and no longer calls `tandem_papercut`, so retiring the adapter is a valid outcome.
