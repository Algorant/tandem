---
id: task-96
uid: c134ee18-5415-4fb3-bc3f-b75390835d63
type: task
title: "Board exclude-tag filter with a saved default view per workspace"
state: todo
priority: "medium"
references: ["task-75"]
tags: ["tui", "config"]
accord:
  status: "ready"
  acceptance: ["The TUI Board can hide every task carrying a chosen tag, and the active filter summary shows the exclusion.", "A workspace can save that exclusion as its Board view in `.tandem/config.toml`, and a fresh `tandem tui` in that workspace opens with it applied and can clear it for the session.", "A plan is approved through Herdra before implementation starts."]
  updatedAt: "2026-10-30T13:35:21Z"
createdAt: "2026-10-30T13:35:21Z"
updatedAt: "2026-10-30T13:35:21Z"
---

## Description

Requested by Algorant via Herdra (m73d1218b). He tags work tasks `#work` on a board and wants a Board view that hides them ("everything except #work").

Today `BoardFilters` (tandem/src/tui/board/mod.rs) holds one optional include `tag` plus kind/priority/delivered-untriaged. It cannot exclude a tag, and filters are per-session only; nothing is saved.

Smallest version:
- An exclude-tag Board filter ("not #work"), alongside the existing include filter, shown in the filter summary.
- A way to persist it as the workspace's default (or named) Board view, likely in workspace `.tandem/config.toml` next to `[board.badges]`.

Capture only: plan first and bring the plan to Herdra before building. Open questions for the plan: one excluded tag or a list; default view only or named views plus a picker; whether CLI `list` and the web view should honor the same config or stay TUI-only; how an active saved view is shown and cleared in-session.
