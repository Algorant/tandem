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
  status: "claimed"
  acceptance: ["Board tabs are ALL, TODO, IN PROGRESS, VALIDATION; the RESEARCH and PAPERCUTS tabs are gone.", "Every tab shows STANDARD, RESEARCH, PAPERCUTS sections with counts, filtered to that tab's state. Empty sections are hidden, and headers are static and skipped by navigation.", "STANDARD keeps the tree, and an Epic's N hidden counts all children. RESEARCH/PAPERCUTS are flat, without kind badges, and show a ↳ parent chip when nested.", "Placing a research or papercut Task under a parent via add or update --parent succeeds and prints a warning recommending a root Task plus `tandem link add`.", "Verified from a release build rendered in a Herdr pane (ANSI read). Tests, docs/tui, and docs/cli are updated, and `just dev-check` and strict clippy pass."]
  claimedAt: "2026-10-02T04:18:39Z"
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-02T04:18:39Z"
createdAt: "2026-10-02T04:01:32Z"
updatedAt: "2026-10-02T04:18:39Z"
assignee: "worker-task-60-311d0ff1"
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