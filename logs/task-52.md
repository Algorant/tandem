---
id: task-52
uid: 497b7bbb-125e-420a-a733-3021ff8e8654
type: task
kind: papercut
title: "CLI reference still documents pre-0.3.0 commands"
priority: "low"
relatedFiles: ["docs/cli/index.md"]
tags: ["docs"]
accord:
  status: "ready"
  acceptance: ["docs/cli/index.md documents exactly the current clap command tree and no removed commands."]
  updatedAt: "2026-10-10T14:34:02Z"
createdAt: "2026-09-30T04:51:58Z"
updatedAt: "2026-10-10T14:34:02Z"
archivedAt: "2026-10-10T14:34:02Z"
links:
  fixed-by: ["task-66"]
resolution:
  outcome: "completed"
  note: "CLI reference rewritten against 0.16.2 in task-66."
---

## Description

Found during task-49. docs/cli/index.md still documents removed surfaces (upgrade, move, log, decision, papercut families, protocol 0.2.0 behavior). task-49 replaced only the checkpoint section with sync and migrate.
