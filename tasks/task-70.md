---
id: task-70
uid: fca842a6-08c8-42c2-857e-fda75e99c656
type: task
kind: "research"
title: "Orchestrator mode: switch the TUI and web view between several Tandem workspaces"
state: "in-progress"
priority: "medium"
tags: ["tui", "ui"]
accord:
  status: "delivered"
  acceptance: ["Sideshow mockups of the TUI switcher popup, the TUI with a switched workspace, any all-workspaces overview, and the web view's equivalent.", "A short findings note answers the five questions (configuration, switcher, overview, web view, architecture cost) with a recommendation for each.", "Proposed follow-up implementation Tasks, each with scope and acceptance criteria, are listed for Algorant to approve."]
  claimedAt: "2026-10-12T14:09:34Z"
  deliveredAt: "2026-10-12T14:13:50Z"
  summary: "Mockups and findings published in Sideshow. Recommendation: an explicit user list in ~/.config/tandem/workspaces.toml; a `w` fuzzy switcher, recent-first; a `W` all-workspaces overview from local reads; one opt-in loopback web server with /w/<alias>/ routes and an /all page; only the active workspace syncs. Costs: list S, TUI switch M, TUI overview M–L, web selector M, web overview M. Five follow-up Tasks are proposed, not filed."
  evidence: ["Sideshow post with TUI switcher, TUI overview and web mockups plus the findings note: http://desktop.tail1cefc.ts.net:8228/session/B_I5vQctbo0 (MagicDNS curl 200)", "Architecture cost from read-only scout orch-mode-arch with file:line evidence: discovery is app/project.rs:24-28 and project/mod.rs:57-100; TUI state and reload are tui/mod.rs:83-93,219-221 and tui/reload.rs:63-297; background sync is app/background_sync.rs:14-96; the web server is web.rs:50-162,464-483; user config is tui/theme.rs:287-296,850-898,1072-1093", "Mockups were rendered in headless Chromium and checked visually before publishing"]
  updatedAt: "2026-10-12T14:13:50Z"
createdAt: "2026-10-12T14:05:13Z"
updatedAt: "2026-10-12T14:13:50Z"
assignee: "tandem-ceph"
---

## Description

From Algorant via Herdra (was herdra task-20). Someone who orchestrates many projects, as Herdra does, picks which Tandem workspaces (repos) are enabled. A small popup switches between them, and the TUI (and the web view) shows the selected workspace's board on the fly.

Questions:
1. **Configuration.** Is the workspace set a user-level list (for example in `~/.config/tandem/config.toml`), discovered from known workspaces, or both?
2. **Switcher.** How it looks and behaves: the key, a fuzzy picker, recent-first ordering, what it shows per workspace (title, counts, attention).
3. **Overview.** Is there an all-workspaces overview (counts, validation and blocked work per workspace)?
4. **Web view.** One server per workspace, or one server with a workspace selector?
5. **Cost.** What it costs inside the current single-workspace architecture: `project::TandemProject` discovery, `app` operations, TUI state and reload, web server binding, and background sync per workspace.

Waits behind the docs overhaul unless Algorant says otherwise.
