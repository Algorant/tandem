---
id: task-22
uid: 5bee27a0-1956-45a8-a7f0-0ea5701cf1ee
type: task
title: "Review modeling of research and papercut tasks without duplicating Board display work"
references: ["task-21", "task-55", "task-56", "task-57", "task-58"]
tags: ["tui", "taxonomy"]
accord:
  status: "ready"
  acceptance: ["Document representative research and papercut workflows and identify differences requiring protocol semantics rather than presentation/filtering.", "Compare ordinary Tasks with tags, task kinds and first-class types, including lifecycle, hierarchy, metadata, interoperability and migration costs.", "Use task-21's agreed overlapping Papercuts view as the display baseline; do not duplicate its counts/navigation implementation or block it on taxonomy choices.", "Recommend a model for each category with rationale and explicitly unresolved product questions, including jointly tagged Tasks where relevant.", "Produce scoped follow-ups only after direction is agreed; make no taxonomy or TUI implementation changes under this exploration."]
  updatedAt: "2026-10-01T22:42:52Z"
createdAt: "2026-09-07T13:41:48Z"
updatedAt: "2026-10-01T22:42:52Z"
archivedAt: "2026-10-01T22:42:52Z"
resolution:
  outcome: "completed"
---
## Goal
Evaluate whether research and papercut workflows need semantics beyond ordinary Tasks with tags. Compare tags, task kinds, and first-class types using concrete lifecycle, hierarchy, metadata, interoperability, and migration needs. Classification must not become workflow state.

## Boundary agreed during papercut cleanup
Algorant selected an overlapping Papercuts tag view with a flat matching list and parent context. Task-21 owns its membership, counts, discoverability, rendering and navigation implementation. Consume that verified behavior as the baseline; do not repeat task-21's display investigation or delay its concrete defect fix for taxonomy research.

Research-specific presentation and tasks tagged both research and papercut may be discussed only where a modeling difference actually requires it. Preserve current task/tag semantics during exploration. No new types, protocol taxonomy changes, or implementation is authorized here.

## Deliverable
A reviewable recommendation for each category, distinguishing actual protocol needs from presentation preferences, with unresolved product choices and scoped implementation follow-ups after direction is agreed.

## Findings: direction agreed with Algorant
- **Research and papercut both need semantics beyond tags.** Kind drives the required fields at `add`, the default priority, the papercut placement rule, and (in Pi) the Worker/Subagent route and batch runs. They become first-class `kind` values alongside `epic`. New first-class *types* and standing container Epics were considered and rejected: they need exceptions to closure, identity, and reparenting rules.
- **Papercut:** title only (acceptance optional), `priority: low` by default, root Task or Epic child only (never a Subtask). Connected to other work through links, not nesting.
- **Research:** title + acceptance, allowed anywhere in the hierarchy.
- **Jointly tagged Tasks:** reported by the migration and not converted; a Task has exactly one kind.
- **Display:** the static TUI gets ALL-first tabs plus RESEARCH/PAPERCUTS kind tabs, built on the existing flat papercut tab rather than duplicating it. The `i` inbox panel is removed. Lanes belong only in `tandem web`.
- **Migration:** active Board only. Archived Logs keep their legacy tags.
- **Follow-ups:**
  - task-55 kinds (Subtasks task-55-1 protocol/CLI, task-55-2 migration, task-55-3 TUI)
  - task-56 typed links
  - task-57 web lanes
  - task-58 pi-tandem adapter
- Discussion and mockups: http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8

