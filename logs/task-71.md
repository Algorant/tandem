---
id: task-71
uid: 6f12d249-3227-4df5-8bf9-57201794b23a
type: task
title: "Orchestrator mode: hand-written workspace list and open_at"
priority: "medium"
effort: "small"
references: ["task-70"]
relatedFiles: ["tandem/src/app/project.rs", "tandem/src/project/mod.rs", "tandem/src/tui/theme.rs"]
tags: ["config", "orchestrator"]
accord:
  status: "accepted"
  acceptance: ["A hand-written user-level workspace list resolves aliases to canonical workspace roots through a shared loader usable by TUI and web.", "open_at(path) opens an explicitly selected board with the same protocol checks as the current entry point.", "Opening tandem tui does not add, rewrite or auto-discover workspace-list entries; invalid entries are reported clearly."]
  claimedAt: "2026-10-12T16:16:42Z"
  deliveredAt: "2026-10-12T16:28:13Z"
  validation: ["just dev-check"]
  summary: "Implemented the read-only, hand-written user workspace list loader, shared app::project::open_at entry point, startup TUI diagnostics, and schema documentation. Committed as de1571d."
  evidence: ["A hand-written user-level workspace list resolves aliases to canonical workspace roots through a shared loader usable by TUI and web.: Added project::workspaces loader at $XDG_CONFIG_HOME/tandem/workspaces.toml or ~/.config/tandem/workspaces.toml. Valid aliases resolve to canonical roots; duplicates, malformed entries, and path errors produce per-entry diagnostics. TUI consumes diagnostics and app exposes the shared loader for web.", "open_at(path) opens an explicitly selected board with the same protocol checks as the current entry point.: app::project::open_at(path) uses project discovery and ensure_current_protocol; temporary checks opened a current board and rejected protocol 0.4.0 and unsupported 9.9.9.", "Opening tandem tui does not add, rewrite or auto-discover workspace-list entries; invalid entries are reported clearly.: PTY smoke with a temporary XDG config showed the invalid second entry as a runtime warning. The list was byte-identical after opening, and no list was created under the board. No switcher, overview, or workspace-management CLI was added."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/README.md", "tandem/src/app/project.rs", "tandem/src/project/mod.rs", "tandem/src/project/workspaces.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/theme.rs"]
  updatedAt: "2026-10-12T16:28:14Z"
createdAt: "2026-10-12T15:51:19Z"
updatedAt: "2026-10-12T16:28:14Z"
assignee: "worker-task-71-6f297b47"
archivedAt: "2026-10-12T16:28:14Z"
resolution:
  outcome: "completed"
---

## Description

Approved by Algorant via Herdra after task-70. Implement a shared loader for ~/.config/tandem/workspaces.toml (honour XDG_CONFIG_HOME), with explicit path and alias entries, and an app::project::open_at(path) entry point preserving protocol checks. The list is hand-written only: opening tandem tui must never auto-add entries. Keep machine-local paths out of the synced board. Missing, duplicate or invalid entries must produce clear per-entry diagnostics. No switcher, overview or workspace-management CLI in this slice. Kickoff approved by Algorant via Herdra: implement in order 71 → 75. Visual Tasks remain delivered for Algorant's review, with actual screenshots in Sideshow. Keep docs overhaul task-76 paused.
