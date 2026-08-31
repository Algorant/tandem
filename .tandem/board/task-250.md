---
id: task-250
type: task
title: "Polish 0.3.0 TUI Papercuts section accessibility and CLI landing page"
state: "in-progress"
priority: "medium"
effort: "small"
references: ["papercut-10", "papercut-11"]
tags: ["tui", "cli", "papercuts"]
createdAt: "2026-08-31T02:40:14Z"
updatedAt: "2026-08-31T02:41:13Z"
accord:
  status: "claimed"
  assignee: "worker-task-250-99bc722a"
  claimedAt: "2026-08-31T02:41:13Z"
  updatedAt: "2026-08-31T02:41:13Z"
assignee: "worker-task-250-99bc722a"
---

## Description

Goal: resolve papercut-10 (Papercuts section cannot be Tab-reached and its section tab is not clickable) and papercut-11 (bare landing page) before live dogfooding of the protocol 0.3.0 cutover.

Scope:
1. Papercuts as a true peer Board section: add the derived __papercuts section to the Board section cycle so Tab / h-l / state-tab keys select it just like Todo/In progress/Validation; register its section-tab mouse hit region so clicking the PAPERCUTS tab selects the section. Reconcile the 'i' inbox popover with section selection (decide: section selection replaces the popover entry, or both coexist; keep 'i' behavior documented).
2. Landing page: restore the old grouped, descriptive landing for the 23-command 0.3.0 surface - grouped sections with one-line command descriptions and the trailing help hint, matched to the generated help names (init; add task|decision; show/list/search/update; accord claim|deliver|rework|block|resume|release|fail; review/complete/cancel; rules list|add|edit|delete; tui; web). No workspace discovery; exit 0.

Constraints: preserve State/Epic Board arrangement, hot reload, themes, mouse model, and all existing behavior; update TUI rendering/interaction tests (four sections, Papercuts tab hit, section cycle) and process tests (landing output shape); render the release TUI in a Herdr pane as evidence (rule always-12) showing Tab reaching Papercuts and the PAPERCUTS tab click selecting the section; keep the 27 generated help surfaces unchanged.

Acceptance: Tab and h/l reach the Papercuts section; clicking the PAPERCUTS section tab selects it (unit + rendered-pane evidence); landing shows grouped commands with descriptions; cargo test, fmt, clippy -D warnings, release build all green; smoke suite still passes.
