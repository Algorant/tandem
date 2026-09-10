---
id: task-30
type: task
title: "TUI Board detail: show a References row"
state: todo
priority: "low"
effort: "small"
references: ["task-29"]
relatedFiles: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/papercuts.rs"]
tags: ["tui"]
accord:
  status: "ready"
  acceptance: ["TUI Board task detail renders a References row listing document IDs and URL values", "Rendering handles a record with no references without adding an empty row", "just dev-check passes"]
  validation: ["$ just dev-check"]
  constraints: ["Display-only plain text References row; no OSC-8 work, validation, storage, or lifecycle changes.", "Keep implementation and rendering regression tests in tandem/src/tui/board/mod.rs; Decisions and Papercuts are styling references, not mutation scope."]
  updatedAt: "2026-09-10T18:00:01Z"
createdAt: "2026-09-10T17:54:53Z"
updatedAt: "2026-09-10T18:00:01Z"
---

## Description

## Description

Follow-up from task-29 (reference links: accept absolute URLs and scope unresolved warnings to the Board).

The TUI Board task detail shows Type, Kind, Role, State, Priority, Effort, Assignee, Due, Tags, and Parent, but never `references` (`tandem/src/tui/board/mod.rs`). The Decisions view and Papercuts panel already render references; Board tasks do not. Once task-29 makes absolute URLs legitimate reference values, the Board detail should expose them too.

Plain text rendering is acceptable; an OSC-8 hyperlink for URL values is optional and must not regress the existing Decisions/Papercuts styling. This row is display-only and must not change validation, storage, or lifecycle behavior.
