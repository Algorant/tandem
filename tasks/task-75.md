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
  validation: ["just dev-check"]
  updatedAt: "2026-10-12T15:51:25Z"
createdAt: "2026-10-12T15:51:25Z"
updatedAt: "2026-10-12T16:16:26Z"
---

## Description

Approved task-70 follow-up. Add an /all overview for opt-in multi-workspace web serving, backed by the shared BoardSummary query. Show attention and local activity with navigation to individual boards. Keep per-entry errors visible. Use bounded local summary refresh instead of calling full load_read for every board on each browser poll. Preserve the explicit configured-board allowlist and existing web security boundary. Kickoff approved by Algorant via Herdra: implement in order 71 → 75. Visual Tasks remain delivered for Algorant's review, with actual screenshots in Sideshow. Keep docs overhaul task-76 paused.
