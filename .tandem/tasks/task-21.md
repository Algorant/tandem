---
id: task-21
type: task
title: "Make Papercuts an overlapping, fully discoverable Board tag view"
state: "in-progress"
priority: "low"
tags: ["tui", "papercut"]
accord:
  status: "claimed"
  acceptance: ["Papercut-tagged active tasks appear in their normal workflow-state views and the dedicated Papercuts view; no exclusive membership partition remains.", "The dedicated Papercuts view lists every matching active record once, including nested matches beneath non-papercut parents, without requiring expansion; parent context distinguishes hierarchy from references.", "Tab counts correspond to matching records under current filters; global totals and filtered totals are visibly distinguished when different; every counted match is discoverable.", "Existing Board State/Epic hierarchy navigation, selection and keyboard/mouse interactions remain correct; flat Papercuts selection opens the correct record and exposes useful parent context.", "Regression and real rendering evidence cover the reproduced4-count/1-row case, hierarchy roles, workflow states, filters, empty lists and normal Board overlap."]
  claimedAt: "2026-09-11T14:34:44Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml --check"]
  constraints: ["TUI display/filtering only: no new document types, task kinds, workflow states, persistence behavior, or protocol reinterpretation.", "Own tandem/src/tui/board/**, state.rs, chrome.rs, mod.rs plus narrow TUI usage docs. Do not change app/CLI/protocol implementation.", "Wait for task-32's overlapping TUI changes to integrate before Worker launch. Submit plan/design before edits and stop for approval."]
  updatedAt: "2026-09-11T14:34:44Z"
createdAt: "2026-09-06T14:20:33Z"
updatedAt: "2026-09-11T14:34:44Z"
references: ["task-22"]
effort: "medium"
relatedFiles: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/state.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/mod.rs", "docs/tui/index.md", "tandem/README.md"]
assignee: "worker-task-21-b150bc53"
---
## Confirmed defect
The0.13.1 audit reproduced a normal parent with three papercut Subtasks plus one standalone papercut: CLI finds four, the Papercuts header/tab says4, but only the standalone row is visible. Expanding the ordinary parent in TODO reveals the other three. Counting scans all tagged documents, while Papercuts row projection excludes a non-papercut ancestor before traversing children.

## Algorant-approved implementation direction
Papercuts are an overlapping tag-based view. They also remain visible in their normal workflow-state tabs. The dedicated Papercuts view is a flat matching list, one row per matching active Task/Epic/Subtask, with parent context where relevant; no required ancestor expansion can hide a counted match. References are loose links, never hierarchy. Preserve task/tag taxonomy and existing normal Board State/Epic behavior outside membership corrections.

Task-22 owns future modeling research and consumes this display baseline rather than duplicating or blocking it. This Task is now explicitly authorized for implementation, replacing its earlier exploration-only scope.

## Validation focus
Exercise standalone, Epic-child, Task-child, cross-state parent/child and referenced-only papercuts; filtered/no-filter counts and no matches; keyboard/mouse selection and context/navigation. Parent will compare rendered release panes against the recorded 4-count/1-row baseline.