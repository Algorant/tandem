# Tandem v0.6.2

Tandem v0.6.2 makes the Board hierarchy easier to scan, navigate, and trust.

## TUI

- Board rows now show compact `#<task-number>` identifiers across State and Epic arrangements, including nested hierarchy rows.
- Every visible State Board row now shows its own workflow state (`TODO`, `WIP`, or `VAL`).
- In-progress Subtasks appear in the In Progress pane with their Epic → Task context revealed, so the counter and visible work agree.
- Same-state hierarchies remain collapsed by default while cross-state work is surfaced in the correct state pane.
- Selecting an Epic now shows a compact descendant-completion progress bar in the Board header, including its completed/total ratio.
- Themes may customize the progress-bar fill through `[colors] progress`.

## Bug fixes

- Fixed Board hierarchy expansion revealing all same-state Epic descendants by default.
- Fixed active Subtasks and direct Epic-child Tasks being difficult to identify in State Board views.
- Fixed small nonzero Epic completion ratios appearing indistinguishable from zero by increasing progress-bar resolution.
