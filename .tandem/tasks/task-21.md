---
id: task-21
type: task
title: "Review TUI Papercuts display, tab membership, and count clarity"
state: todo
priority: "low"
tags: ["tui", "papercut"]
accord:
  status: "ready"
  acceptance: ["Reproduce the reported four-count/one-visible-row mismatch and either-Papercuts-or-TODO behavior, or document the attempted cases and exact missing data needed to reproduce them.", "Trace count, tab membership, and displayed-row selection paths and identify whether hierarchy, references, filters, collapse state, or exclusive tab partitioning explain the mismatch; distinguish confirmed causes from hypotheses.", "Evaluate standalone, Epic-child, Task-child, and loosely related papercuts, including matches beneath non-papercut parents, and record expected versus observed counts and visibility across Papercuts and workflow-state tabs.", "Compare overlapping tag-based views with exclusive tab membership and recommend explicit display semantics; define global totals versus tab/visible counts and how any difference is clearly communicated.", "Propose a navigable display in which every counted papercut is discoverable; describe hierarchy/context and keyboard/mouse implications, coordinate with task-22, and surface unresolved UX choices.", "Record a scoped implementation follow-up with regression-test scenarios and rendering validation criteria; do not silently expand this exploratory task into implementation."]
  updatedAt: "2026-09-07T13:55:21Z"
createdAt: "2026-09-06T14:20:33Z"
updatedAt: "2026-09-07T13:55:21Z"
references: ["task-22"]
---
## Reported symptoms
Papercut-tagged tasks sometimes do not appear in the TUI Papercuts view when nested beneath other tasks or related to them, while the papercut count continues to increase. Nesting/relationships are a suspected trigger, not a confirmed cause.

Additional user observation: papercuts appear either in the Papercuts tab or in the TODO tab, but not both. The Papercuts tab/header nevertheless includes the total, including papercuts displayed elsewhere. This makes counts misleading and leaves unclear where a user should look for a given papercut. Treat this as reported behavior to verify, not a confirmed implementation diagnosis.

## Screenshot evidence
The user-provided screenshot shows both the top-right `Papercuts 4` indicator and selected `PAPERCUTS 4` tab, but only one visible row: task-58, `Remove inactive legacy rules block through supported Tandem operation` (TODO, LOW). The remaining pane is empty. This is an example from the user's .pi workspace, not a claim about task-58 in the Tandem repository.

## Exploration scope
Review Papercuts display generally, including membership across the Papercuts and workflow-state tabs. Reproduce and trace how counts, filtering, hierarchy projection, collapsed parents, and row visibility interact. Compare standalone papercuts, Tasks beneath Epics, Subtasks beneath Tasks, and loosely referenced/related tasks; distinguish actual parent relationships from references. Check whether non-papercut parents, workflow state, or active filters hide matching descendants.

Determine whether Papercuts should be an overlapping tag-based view (tasks also appear in their workflow-state tabs) or an exclusive partition. Evaluate both without assuming either is already the intended design. Define what each tab count and the global Papercuts indicator count, how totals differ from visible matches, and how any difference is explained to users.

Explore presentation options that make counted papercuts discoverable and navigable without silently dropping nested matches: for example a flat matching list with parent context versus a hierarchy retaining necessary ancestors. These are options, not a selected design. Recommend clear membership, count semantics, and navigation/display behavior, and capture a bounded implementation follow-up. Coordinate with task-22's broader research/papercut modeling review. This task authorizes investigation and a proposal, not implementation.