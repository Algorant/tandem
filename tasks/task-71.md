---
id: task-71
uid: 6f12d249-3227-4df5-8bf9-57201794b23a
type: task
title: "Orchestrator mode: hand-written workspace list and open_at"
state: todo
priority: "medium"
effort: "small"
references: ["task-70"]
relatedFiles: ["tandem/src/app/project.rs", "tandem/src/project/mod.rs", "tandem/src/tui/theme.rs"]
tags: ["config", "orchestrator"]
accord:
  status: "ready"
  acceptance: ["A hand-written user-level workspace list resolves aliases to canonical workspace roots through a shared loader usable by TUI and web.", "open_at(path) opens an explicitly selected board with the same protocol checks as the current entry point.", "Opening tandem tui does not add, rewrite or auto-discover workspace-list entries; invalid entries are reported clearly."]
  validation: ["just dev-check"]
  updatedAt: "2026-10-12T15:51:19Z"
createdAt: "2026-10-12T15:51:19Z"
updatedAt: "2026-10-12T16:16:21Z"
---

## Description

Approved by Algorant via Herdra after task-70. Implement a shared loader for ~/.config/tandem/workspaces.toml (honour XDG_CONFIG_HOME), with explicit path and alias entries, and an app::project::open_at(path) entry point preserving protocol checks. The list is hand-written only: opening tandem tui must never auto-add entries. Keep machine-local paths out of the synced board. Missing, duplicate or invalid entries must produce clear per-entry diagnostics. No switcher, overview or workspace-management CLI in this slice. Kickoff approved by Algorant via Herdra: implement in order 71 → 75. Visual Tasks remain delivered for Algorant's review, with actual screenshots in Sideshow. Keep docs overhaul task-76 paused.
