---
id: task-55-3
uid: 0a46cb4f-8d47-4433-8a31-2c99ab626085
type: task
title: "TUI: ALL-first Board tabs, kind tabs and badges, remove papercut inbox"
state: todo
effort: "medium"
parentId: "task-55"
relatedFiles: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/papercuts.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/state.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/input.rs"]
tags: ["tui", "taxonomy", "keyboard"]
accord:
  status: "ready"
  acceptance: ["Board tabs are ALL (first and the default) · each workflow state · RESEARCH · PAPERCUTS. ALL and the state tabs keep the existing tree; the kind tabs are flat, cross-state lists that show parent context.", "The synthetic `__papercuts` state and its index special cases are replaced by one small set of view types: All / State / Kind. Selection survives reload in every tab.", "RESEARCH and PAPERCUT badges come from `kind`; the `f` filter can filter by kind; the built-in `research` tag badge is removed.", "The `i` papercut inbox panel (`papercuts.rs`), its header indicator, and its key/mouse handling are removed.", "The rendered result is verified in a Herdr pane per rule always-4."]
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-01T22:20:50Z"
createdAt: "2026-10-01T22:20:50Z"
updatedAt: "2026-10-01T22:20:50Z"
---

## Description

Static TUI only: no stacked lanes, collapse, or run hints (those belong in tandem web). The target tab layout is in the mockup post "Draft outline" at http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8.
