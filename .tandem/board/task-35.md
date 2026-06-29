---
id: task-35
type: task
title: "Differentiate research and spike tasks in the TUI"
state: "validation"
priority: "medium"
relatedFiles: ["tandem/src/tui.rs", "protocol/plan/spec.md", "tandem/plan/spec.md"]
tags: ["tui", "research", "ux"]
references: ["task-24"]
createdAt: "2026-06-28T16:17:06Z"
updatedAt: "2026-06-29T00:20:10Z"
subtasks:
  - id: task-35-1
    title: "Use tag-driven research/spike badges for v0 rather than introducing a new protocol type"
    completed: false
  - id: task-35-2
    title: "Render research/spike tasks distinctly in Board rows/details without reintroducing noisy default task type metadata"
    completed: false
  - id: task-35-3
    title: "Document the convention so research findings can live directly on tasks when a separate memo is unnecessary"
    completed: false
  - id: task-35-4
    title: "Add focused TUI tests or snapshots for the visual distinction"
    completed: false
accord:
  status: "delivered"
  deliveredAt: "2026-06-29T00:20:10Z"
  summary: "Implemented TUI Board research/spike differentiation using existing tags. Board rows now surface RESEARCH/SPIKE chips for tasks tagged research or spike, with focused tests covering task-24-style research examples and spike rows."
  evidence: ["cd tandem && cargo fmt --check && cargo test (passed: 63 tests)", "Commit fccda7d on branch herd-task35-52-board-ux"]
  filesChanged: ["tandem/src/tui.rs"]
  updatedAt: "2026-06-29T00:20:10Z"
---

## Description

Research/spike tasks should be easy to distinguish from implementation tasks in the Board and details. For v0, use existing tags such as `research` or `spike` to drive a lightweight visual badge/convention rather than introducing `type: research` or other new protocol machinery. Keep v0 compatible with existing task documents and avoid forcing research into separate memo files.

Related context: `task-24` is a docs-platform research task and should be one of the examples used to validate the convention.
