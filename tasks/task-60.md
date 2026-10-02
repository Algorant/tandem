---
id: task-60
uid: 0a1a71af-9885-4233-b0ab-675209d7038e
type: task
title: "TUI Board: state-only tabs with kind sections in every tab; warn on nested research/papercut"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["task-55", "task-57"]
relatedFiles: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/app/tasks.rs", "docs/tui/index.md", "docs/cli/index.md"]
tags: ["tui", "taxonomy"]
accord:
  status: "delivered"
  acceptance: ["Board tabs are ALL, TODO, IN PROGRESS, VALIDATION; the RESEARCH and PAPERCUTS tabs are gone.", "Every tab shows STANDARD, RESEARCH, PAPERCUTS sections with counts, filtered to that tab's state. Empty sections are hidden, and headers are static and skipped by navigation.", "STANDARD keeps the tree, and an Epic's N hidden counts all children. RESEARCH/PAPERCUTS are flat, without kind badges, and show a ↳ parent chip when nested.", "Placing a research or papercut Task under a parent via add or update --parent succeeds and prints a warning recommending a root Task plus `tandem link add`.", "Verified from a release build rendered in a Herdr pane (ANSI read). Tests, docs/tui, and docs/cli are updated, and `just dev-check` and strict clippy pass."]
  claimedAt: "2026-10-02T04:33:59Z"
  deliveredAt: "2026-10-02T04:43:48Z"
  validation: ["$ just dev-check"]
  summary: "Board tabs are now ALL · TODO · IN PROGRESS · VALIDATION, and the Kind tabs and BoardView::Kind are removed. Every tab shows STANDARD, RESEARCH and PAPERCUTS sections filtered to the tab's state, with counts. Empty sections are hidden, and headers are static and skipped by j/k. add and update --parent still allow a nested research or papercut Task, and now warn. The work is committed as c519eba. No gaps remain. Live-terminal tab redraw and mouse clicks were not checked."
  evidence: ["Board tabs are ALL, TODO, IN PROGRESS, VALIDATION; RESEARCH and PAPERCUTS tabs are gone: Rendered tab bar: `ALL 9 │ TODO 6 │ IN PROGRESS 3 │ VALIDATION 0`. BoardView is now {All, State}. KIND_VIEWS, BoardView::Kind, is_flat and kind_board_entries are deleted. The ANSI read shows ALL as the selected tab: bold, underlined, green (38;2;142;192;124), with the other tabs in muted grey (146;131;116).", "Every tab shows STANDARD, RESEARCH, PAPERCUTS sections with counts, filtered to the tab's state. Empty sections hidden; headers static and skipped by navigation: ALL showed `STANDARD 5`, `RESEARCH 2`, `PAPERCUTS 2` (sums to the ALL 9 tab count). TODO showed STANDARD 3, RESEARCH 1, PAPERCUTS 2. IN PROGRESS showed STANDARD 2 and RESEARCH 1, with PAPERCUTS hidden. VALIDATION with no tasks showed `No active items in this state.` The ANSI read shows headers as bold (SGR 1) grey text followed by a muted count, for example `STANDARD`, `RESEARCH`, `PAPERCUTS`. On ALL, j moved the selection #1→#3→#4→#9→#5, jumping the RESEARCH header, and later reached #8 past the PAPERCUTS header. Unit tests also check that mouse hits skip headers.", "STANDARD keeps the tree, and an Epic's N hidden counts all children. RESEARCH/PAPERCUTS are flat, without kind badges, and show a ↳ parent chip when nested: Collapsed Epic #1 showed `3 matches hidden` on ALL, counting one standard child plus a nested research and a nested papercut. On TODO it showed `2 matches hidden` and on IN PROGRESS `1 match hidden`. After Enter it showed `▾ … 3 active`, with only #2 `└─` beneath it. Nested #6 and #8 show `↳ Task of Epic: Web interf…`. Root #5 and #7 show no chip. Rows in RESEARCH and PAPERCUTS have no RESEARCH or PAPERCUT badge, while the Epic keeps its EPIC chip. Unit tests cover the nesting rules.", "Placing a research or papercut Task under a parent via add or update --parent succeeds and prints a warning recommending a root Task plus `tandem link add`: `add task ... --kind research --parent task-1` created the Task and printed `Warning: a research Task under task-1 is allowed but not recommended; prefer a root Task plus `tandem link add <id> relates-to task-1``. The papercut case is the same. The integration test nesting_research_or_papercut_warns_and_recommends_a_link checks the JSON `warnings` array and the stderr text for add, the real ID on update --parent, and no warning for a standard Subtask or a non-parent update.", "Verified from a release build rendered in a Herdr pane (ANSI read); tests, docs/tui and docs/cli updated; just dev-check and strict clippy pass: See validation. docs/tui/index.md and docs/cli/index.md are updated, along with the tandem/README.md Board paragraph. The historical RELEASES.md entry and tandem/plan/spec.md are unchanged."]
  filesChanged: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/input.rs", "tandem/src/tui/mod.rs", "tandem/src/app/tasks.rs", "tandem/src/cli/commands.rs", "tandem/tests/kind_behavior.rs", "docs/tui/index.md", "docs/cli/index.md", "tandem/README.md"]
  updatedAt: "2026-10-02T04:43:48Z"
createdAt: "2026-10-02T04:01:32Z"
updatedAt: "2026-10-02T04:43:48Z"
assignee: "worker-task-60-12d511c8"
---
Agreed with Algorant on 2026-10-02 after the 0.16.1 cutover. The mockup is post 0w4eWzghXSs in the Sideshow session http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8. Keep it simple: no legacy compatibility, and no options beyond what is listed here.

## TUI
- Board tabs: ALL · TODO · IN PROGRESS · VALIDATION (the workflow states). Remove the RESEARCH and PAPERCUTS tabs and the `BoardView::Kind` path behind them.
- Every tab has the same body: sections STANDARD, RESEARCH, PAPERCUTS, in that order, filtered to the tab's state (ALL = every state). Each section header shows a count. An empty section is hidden. Headers are static labels: not selectable, not collapsible, and skipped by `j`/`k`.
- STANDARD (no kind, plus Epics) keeps today's tree, collapse, and `N hidden`. An Epic's `N hidden` counts all its children, including research/papercut children shown in the other sections.
- RESEARCH and PAPERCUTS are flat lists. The kind badge is dropped inside its own section. A Task with a parent shows the existing `↳ parent` context chip. Classification comes from `kind` only.
- Sort (`s`) and filters (`f`) apply within sections. Selection and reload-by-id work across sections.

## CLI
- `add` and `update --parent` still allow a research or papercut Task under a parent, and existing papercut placement rules still hold. They print a warning recommending a root Task plus `tandem link add <id> relates-to <parent>`.

## Unchanged
`tandem web`, the protocol (0.5.0), existing records. No migration.