---
id: task-80
type: task
kind: epic
title: "Add lightweight Epic support as a task kind"
priority: "high"
relatedFiles: ["protocol/plan/spec.md", "tandem/plan/spec.md", "extensions/pi-tandem/plan/spec.md", "README.md"]
tags: ["protocol", "epic", "tasks", "cli", "tui", "docs", "extensions"]
createdAt: "2026-07-01T17:26:15Z"
updatedAt: "2026-07-04T11:36:30Z"
subtasks:
  - id: task-80-1
    title: "Specify optional task kind field with epic semantics"
    completed: false
  - id: task-80-2
    title: "Add CLI creation/display/update behavior for kind: epic"
    completed: false
  - id: task-80-3
    title: "Render epic badge and relationship hints in the TUI"
    completed: false
  - id: task-80-4
    title: "Update docs and extension guidance with epic examples"
    completed: false
completedAt: "2026-07-04T11:36:30Z"
completion:
  summary: "Completed parent Epic support task after validating that all child implementation, TUI, documentation, and agent guidance work has been completed and accepted."
  validation: "User requested marking task-80 validated/complete. Child work completed: task-82 Epic protocol/CLI core, task-83 Epic TUI rendering and relationship hints with visual approval, and task-84 Epic documentation/agent guidance."
---

## Description

Add minimal epic support without introducing a new first-class document type or heavy project-management system.

Agreed direction:
- Model epics as task documents with `type: task` and `kind: epic`.
- Omitted `kind` continues to mean a normal task.
- Epics remain board-visible and use normal workflow `state` values.
- Child work links to an epic through existing `parentId` relationships; loose associations continue to use `references`.
- Do not add a separate `tandem epic` command, `epic-N` ID allocator, or dedicated Epic pane in the first pass unless later explicitly chosen.

Expected changes:
- Protocol/spec: define the optional task `kind` field and initial supported value `epic`; clarify relationship semantics and completion/archive expectations for epic tasks.
- CLI: allow creation/update/display of epic kind, render an EPIC badge or kind marker in list/show/search where useful, and avoid treating epic as a custom type.
- TUI: render epic tasks with an EPIC badge and optionally child counts/relationship hints using derived `parentId` data.
- Docs/extensions: document the convention for humans and agents, including examples such as a docs-launch epic with child tasks.
