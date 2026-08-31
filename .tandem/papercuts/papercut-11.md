---
id: papercut-11
title: "CLI: bare landing page lacks the old grouped command surface"
status: "resolved"
createdAt: "2026-08-31T02:40:00Z"
updatedAt: "2026-08-31T02:58:09Z"
resolution:
  note: "fixed on main by task-250 (squash 357f510)"
  resolvedAt: "2026-08-31T02:58:09Z"
---
The 0.3.0 landing page (landing.rs) is a three-line summary:

tandem - Tandem CLI 0.3.0
Commands: init, add task|decision, show, list, search, update
  accord claim|deliver|rework|block|resume|release|fail
  review, complete, cancel, rules list|add|edit|delete, tui, web
Run 'tandem <command> --help' for detailed usage.

The 0.11.0 landing was grouped and descriptive (Work/Collaborate/Explore/Workspace sections with a one-line description per command and the trailing --help hint). Restore a grouped landing for the 23-command 0.3.0 surface: grouped sections, one-line descriptions, no workspace discovery, honoring D58 (concise landing, TUI stays explicit).
