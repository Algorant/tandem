---
id: task-60
uid: 0a1a71af-9885-4233-b0ab-675209d7038e
type: task
title: "TUI ALL tab: split rows into Standard, Research, and Papercuts sections"
state: todo
priority: "medium"
effort: "medium"
references: ["task-55", "task-57"]
relatedFiles: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/mod.rs", "docs/tui/index.md"]
tags: ["tui", "taxonomy"]
accord:
  status: "ready"
  acceptance: ["The TUI ALL tab shows Standard, Research, and Papercuts sections in that order, each with a header and count, classified from `kind` only.", "Standard keeps the existing tree; research and papercut Tasks nested under another parent appear in their own section with parent context.", "An empty section shows only its header. Selection, navigation, filters, sort, and reload-by-id work across sections; state and kind tabs are unchanged.", "Verified from a release build rendered in a Herdr pane (ANSI read), with tests and docs/tui updated."]
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-02T04:01:32Z"
createdAt: "2026-10-02T04:01:32Z"
updatedAt: "2026-10-02T04:01:32Z"
---

## Description

Requested by Algorant after the 0.16.1 cutover. The point of the ALL tab is the kind split. Today, ALL shows one interleaved tree, so papercuts and standard work are mixed by ID. This is independent of web lanes (task-57) and keeps the TUI static: the tabs, their order, and the state/kind tabs are unchanged.

The ALL tab renders three titled sections in order: Standard (no kind, plus Epics), Research, Papercuts. Each section header shows a count. Standard keeps the existing parent/child tree. Research and Papercut rows that sit under another parent appear in their own section with parent context (the same `↳` chip as the kind tabs), not nested inside Standard. Classification comes from `kind` only.
