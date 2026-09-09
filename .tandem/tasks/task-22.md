---
id: task-22
type: task
title: "Review modeling and TUI presentation of research and papercut tasks"
state: todo
references: ["task-21"]
tags: ["tui", "taxonomy"]
accord:
  status: "ready"
  acceptance: ["Document representative research and papercut workflows and identify which differences require protocol semantics versus presentation/filtering only.", "Compare ordinary Tasks with tags, a task-kind classification, and first-class types, including lifecycle, hierarchy, metadata, interoperability, and migration costs; do not treat classification as workflow state.", "Evaluate TUI presentation options with explicit behavior for counts, nested tasks, parent context, navigation, and tasks tagged both research and papercut, incorporating task-21 findings when available.", "Recommend a model and TUI approach for each category, with rationale and unresolved product questions clearly separated from settled decisions.", "Produce a reviewable proposal and scoped implementation follow-ups once direction is agreed; no protocol taxonomy or TUI implementation changes are made as part of this exploration."]
  updatedAt: "2026-09-07T13:41:48Z"
createdAt: "2026-09-07T13:41:48Z"
updatedAt: "2026-09-07T13:41:48Z"
---

## Description

## Goal
Review research-tagged and papercut-tagged tasks as distinct user workflows. Determine whether either now warrants a standalone task type/classification, or whether ordinary Tasks with tags and a dedicated interface/filter remain sufficient.

## Questions to explore
- What meaning, lifecycle, required metadata, hierarchy behavior, and actions actually differ from ordinary Tasks for research and papercuts?
- Are tags sufficient, would a task kind be useful, or is a first-class document/task type justified? Distinguish classification from workflow state rather than assuming a new status column is needed.
- How should these tasks appear in the TUI: ordinary Board entries, dedicated views/tabs, saved or special filters, or a combination?
- How should counts, nested matches, parent context, navigation, and tasks carrying both tags behave?
- What complexity, protocol changes, migration, and CLI/context-consumer consequences would each option introduce?

Compare options against concrete workflows and the smallest useful model. Preserve the current tag-based model during exploration; do not introduce new types or implement UI changes under this task. Coordinate with task-21, which investigates the narrower Papercuts count/visibility mismatch, without duplicating that investigation.

## Deliverable
An evidence-backed recommendation for both modeling and TUI presentation, identifying whether research and papercuts should share an approach or differ. Surface product choices for review and capture bounded implementation tasks after direction is agreed.
