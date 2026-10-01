---
id: task-55-1
uid: ade9decf-3f38-47e4-a7d2-3d40427d42f9
type: task
title: "Protocol and CLI: research and papercut kinds, --kind, minimum fields, papercut placement"
state: todo
effort: "medium"
parentId: "task-55"
relatedFiles: ["protocol/README.md", "tandem/src/protocol/document.rs", "tandem/src/protocol/hierarchy.rs"]
tags: ["protocol", "taxonomy"]
accord:
  status: "ready"
  acceptance: ["`TASK_KINDS` accepts `epic`, `research`, and `papercut`; `add` and `update` accept `--kind`, and `list` and `search` filter by `--kind`.", "`add --kind papercut` requires only a title (acceptance optional) and defaults to `priority: low` unless one is given; task and research keep their current required fields.", "A papercut may be a root Task or a direct child of an Epic; creating, reparenting, or re-kinding one into a Subtask fails with a clear validation error.", "The protocol README and CLI reference document the kinds, the papercut defaults, and the placement rule, with tests covering each rule."]
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-01T22:20:48Z"
createdAt: "2026-10-01T22:20:48Z"
updatedAt: "2026-10-01T22:20:48Z"
---

## Description

See task-55 for the agreed decisions. Mockups: http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8
