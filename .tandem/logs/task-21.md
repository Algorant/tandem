---
id: task-21
type: task
title: "Make Papercuts an overlapping, fully discoverable Board tag view"
priority: "low"
tags: ["tui", "papercut"]
accord:
  status: "accepted"
  acceptance: ["Papercut-tagged active tasks appear in their normal workflow-state views and the dedicated Papercuts view; no exclusive membership partition remains.", "The dedicated Papercuts view lists every matching active record once, including nested matches beneath non-papercut parents, without requiring expansion; parent context distinguishes hierarchy from references.", "Tab counts correspond to matching records under current filters; global totals and filtered totals are visibly distinguished when different; every counted match is discoverable.", "Existing Board State/Epic hierarchy navigation, selection and keyboard/mouse interactions remain correct; flat Papercuts selection opens the correct record and exposes useful parent context.", "Regression and real rendering evidence cover the reproduced4-count/1-row case, hierarchy roles, workflow states, filters, empty lists and normal Board overlap."]
  claimedAt: "2026-09-11T14:34:44Z"
  deliveredAt: "2026-09-11T15:05:02Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml --check"]
  constraints: ["TUI display/filtering only: no new document types, task kinds, workflow states, persistence behavior, or protocol reinterpretation.", "Own tandem/src/tui/board/**, state.rs, chrome.rs, mod.rs plus narrow TUI usage docs. Do not change app/CLI/protocol implementation.", "Wait for task-32's overlapping TUI changes to integrate before Worker launch. Submit plan/design before edits and stop for approval."]
  summary: "Ready. Fixed the live reload navigation defect on the flat Papercuts lens. Reload captured the selected Board subview with `self.states.get(self.selected_state)`, which is None for the pseudo Papercuts tab, so restore fell back to the selected record's own workflow state and jumped the subview to Todo. reload.rs now captures an explicit `board_papercuts` flag (guarded on a non-empty state list so the first load still defaults to the first real state) and restores the lens directly: it keeps the selected id when it is still a lens match, otherwise clamps to a remaining match or an empty list. The else branch still uses the original `select_document_by_id_preserving_scroll` behavior, now resolving the captured real state through `board_subview_tabs`, so normal State/Epic selection restoration is unchanged; task-32's rules_warnings capture is untouched. New regression test covers manual reload, auto reload on an external edit to another match, untagging the selected record, and an active filter that empties the lens. Live verification via a Herdr release pane on a fixture copy: with task-1-3 selected on Papercuts, an external edit to task-1-1 triggered auto reload in ~500ms and the pane stayed on PAPERCUTS with task-1-3 selected; untagging task-1-3 kept PAPERCUTS active with count 3 and clamped selection to task-1-1. Declared validations pass: just dev-check (288 bin + 34 integration tests, smoke PASS) and cargo fmt --check."
  evidence: ["Reload preserves the Papercuts lens and existing matching selected id: reload.rs captures board_papercuts = !states.is_empty() && selected_state == states.len() and restores the lens directly. papercut_lens_survives_manual_and_auto_reload_without_jumping_subviews asserts PAPERCUTS + task-11 across a manual reload and an auto reload triggered by an external edit to task-10; the Herdr release pane showed 'Selected task-1-3' and PAPERCUTS 4 before and after an external task-1-1 edit.", "Removed/untagged/filtered-out selected record stays in the lens and clamps: restore_papercuts_lens_selection selects the same id when still present, else selected_item 0 then clamp_selection. Test asserts untagging task-11 keeps PAPERCUTS and selects task-10; a priority filter that empties the lens keeps PAPERCUTS with selected_state_count 0 and no selected doc. Live pane: untagging task-1-3 kept PAPERCUTS 3 and selected task-1-1.", "Normal State/Epic selection restoration does not regress: The non-lens branch is unchanged apart from resolving the captured real state through board_subview_tabs. reload_preserves_selected_document_by_id_after_external_state_change, state_board_reload_preserves_expansion_and_selected_child, and the workflow picker tests all pass. The first-load regression (opening Papercuts) is fixed by the non-empty state guard.", "task-32 rules_warnings capture preserved: reload() still builds and assigns self.rules_warnings from the rules load before restore; only capture/restore selection changed. Full test suite including the rules warning tests passes.", "Declared validations pass on the final commit: just dev-check exit 0 with 288 bin and 34 integration tests passing and both smoke assertions PASS; cargo fmt --check exit 0 with no output, run after the final reload.rs/state.rs/mod.rs edits."]
  filesChanged: ["tandem/src/tui/reload.rs", "tandem/src/tui/state.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "docs/tui/index.md", "tandem/README.md"]
  updatedAt: "2026-09-11T15:05:02Z"
createdAt: "2026-09-06T14:20:33Z"
updatedAt: "2026-09-11T15:05:02Z"
references: ["task-22"]
effort: "medium"
relatedFiles: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/state.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "docs/tui/index.md", "tandem/README.md"]
assignee: "worker-task-21-b150bc53"
archivedAt: "2026-09-11T15:05:02Z"
resolution:
  outcome: "completed"
---
## Confirmed defect
The0.13.1 audit reproduced a normal parent with three papercut Subtasks plus one standalone papercut: CLI finds four, the Papercuts header/tab says4, but only the standalone row is visible. Expanding the ordinary parent in TODO reveals the other three. Counting scans all tagged documents, while Papercuts row projection excludes a non-papercut ancestor before traversing children.

## Algorant-approved implementation direction
Papercuts are an overlapping tag-based view. They also remain visible in their normal workflow-state tabs. The dedicated Papercuts view is a flat matching list, one row per matching active Task/Epic/Subtask, with parent context where relevant; no required ancestor expansion can hide a counted match. References are loose links, never hierarchy. Preserve task/tag taxonomy and existing normal Board State/Epic behavior outside membership corrections.

Task-22 owns future modeling research and consumes this display baseline rather than duplicating or blocking it. This Task is now explicitly authorized for implementation, replacing its earlier exploration-only scope.

## Validation focus
Exercise standalone, Epic-child, Task-child, cross-state parent/child and referenced-only papercuts; filtered/no-filter counts and no matches; keyboard/mouse selection and context/navigation. Parent will compare rendered release panes against the recorded 4-count/1-row baseline.