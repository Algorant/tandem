---
id: task-75
uid: 197f29a1-27bb-4e18-b5ab-c5ed42e996a9
type: task
title: "Deferred: web orchestration over the session roster and BoardSummary"
state: todo
priority: "medium"
effort: "medium"
references: ["task-70", "task-73", "task-74"]
relatedFiles: ["tandem/src/web.rs", "tandem/src/web/app.js", "tandem/src/web/api.js", "tandem/src/app/queries.rs"]
tags: ["ui", "orchestrator"]
accord:
  status: "blocked"
  acceptance: ["In opt-in multi-workspace mode an /all page lists configured boards, summaries, attention and errors and links to their individual views.", "The overview reuses BoardSummary rather than implementing different counting semantics or parsing records in the frontend.", "Polling uses bounded local summaries and does not fetch or fully reload all boards each cycle.", "Single-workspace defaults and the configured-board allowlist remain unchanged."]
  validation: ["$ just dev-check"]
  constraints: ["Deferred: no implementation, Worker launch, server, web selector or web overview until separately approved."]
  note: "Algorant froze orchestrator-mode implementation pending approved ad-hoc ephemeral-workspace plumbing; permanent workspaces.toml is no longer the requirement. Preserve existing work unchanged; no new code/mockups/merges/pushes/discards/rollbacks."
  updatedAt: "2026-10-12T17:32:30Z"
createdAt: "2026-10-12T15:51:25Z"
updatedAt: "2026-10-12T17:32:30Z"
---
## Deferred web orchestration

Algorant's revised direction is TUI first. This replaces the earlier immediate web-overview implementation and all stale freeze instructions. No web feature work now; leave this Task parked until a separate explicit go-ahead.

Future scope may reuse the session roster, open-by-path and BoardSummary interfaces from tasks 73/74 for a closely related web orchestration view. The former task-73 web-selector scope is not being implemented elsewhere now. Revisit opt-in serving, exposure boundaries and exact acceptance before delegation; no mandatory permanent workspace-list file.

This is explicit product deferral, not a dependency that should hold the TUI implementation or docs forever.