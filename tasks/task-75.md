---
id: task-75
uid: 197f29a1-27bb-4e18-b5ab-c5ed42e996a9
type: task
title: "Orchestrator mode: web all-workspaces overview"
state: todo
priority: "medium"
effort: "medium"
blockers: ["task-74"]
references: ["task-70"]
relatedFiles: ["tandem/src/web.rs", "tandem/src/web/app.js", "tandem/src/web/api.js", "tandem/src/app/queries.rs"]
tags: ["ui", "orchestrator"]
accord:
  status: "ready"
  acceptance: ["In opt-in multi-workspace mode an /all page lists configured boards, summaries, attention and errors and links to their individual views.", "The overview reuses BoardSummary rather than implementing different counting semantics or parsing records in the frontend.", "Polling uses bounded local summaries and does not fetch or fully reload all boards each cycle.", "Single-workspace defaults and the configured-board allowlist remain unchanged."]
  validation: ["$ just dev-check"]
  constraints: ["Frozen: revised ephemeral-workspace plumbing/functions require Algorant's approval before any further code, mockups, integration or push. Preserve current work; no discards/rollbacks."]
  updatedAt: "2026-10-12T17:18:09Z"
createdAt: "2026-10-12T15:51:25Z"
updatedAt: "2026-10-12T17:18:09Z"
---
## Frozen by Algorant via Herdra

Do not implement, merge, push, create implementation Workers, produce mockups, discard, clean up or roll back existing work until Algorant approves revised plumbing/functions. Preserve all commits, checkouts and current preview state. New requirement: no dedicated permanent workspaces.toml; enabled/viewable workspaces must be ad-hoc, ephemeral and easy to change during a session. Discuss session roster, on-demand native Herdr project discovery and a repo picker first; no UI/button design yet. w/W remain tentative subject to conflicts. This direction overrides the earlier permanent-list implementation scope below, which is retained as history, not renewed approval.

Task-71 already landed as 1bfc31c and is archived accepted; its code is preserved, not rolled back. Task-72's retained unmerged Worker is clean at 4f6d5e7. Main retains temporary justfile/scripts/dev.sh preview helper mirrors; do not restore them while the no-rollback hold applies.

## Previous scope (on hold)


## Description

Approved task-70 follow-up. Add an /all overview for opt-in multi-workspace web serving, backed by the shared BoardSummary query. Show attention and local activity with navigation to individual boards. Keep per-entry errors visible. Use bounded local summary refresh instead of calling full load_read for every board on each browser poll. Preserve the explicit configured-board allowlist and existing web security boundary. Kickoff approved by Algorant via Herdra: implement in order 71 → 75. Visual Tasks remain delivered for Algorant's review, with actual screenshots in Sideshow. Keep docs overhaul task-76 paused.
