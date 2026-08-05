---
id: task-219
type: task
title: "Add web refresh and local security safeguards"
priority: "high"
parentId: "task-216"
blockers: ["task-217", "task-218"]
references: ["task-121"]
relatedFiles: ["tandem/src"]
tags: ["ui", "web", "security", "refresh"]
createdAt: "2026-08-05T18:46:23Z"
updatedAt: "2026-08-05T19:23:36Z"
accord:
  status: "accepted"
  assignee: "worker-task-219-0596f986"
  claimedAt: "2026-08-05T19:13:50Z"
  deliveredAt: "2026-08-05T19:23:27Z"
  deliverables: ["Three-second visible-tab opaque-revision polling with canonical refetch only on change.", "Exact loopback Host validation plus GET/HEAD-only, no-body, request-target, and concurrency safeguards.", "CSP, frame denial, no-sniff, no-referrer, permissions, no-store, and no-CORS behavior across responses."]
  validation:
    commands: ["Post-integration 240 unit and 11 integration tests passed.", "Formatting, strict all-feature Clippy, release build, JS syntax, and diff checks passed.", "Parent live smoke verified loopback startup, security headers, no CORS, invalid Host rejection, and POST rejection."]
  summary: "Added visible-tab revision polling, changed-view refresh with transient-state preservation, strict Host validation, read-only request limits, restrictive browser headers, and safe error/content handling."
  evidence: ["Integrated commit a2a0abb into main via Worktrunk.", "Reviewed polling state flow, safe Markdown handoff, middleware ordering, request limits, headers, and focused regression tests.", "No SSE, WebSockets, accounts, remote binding, mutation routes, database, or sync behavior was added."]
  filesChanged: ["tandem/src/web.rs", "tandem/src/web/app.js", "tandem/src/web/ui.js", "tandem/src/web/app.css"]
  reviewer: "orchestrator"
  note: "Accepted after integrated security/polling code review, full automated validation, and live Host/header/method/no-CORS smoke. Scope remains intentionally local and read-only."
  updatedAt: "2026-08-05T19:23:31Z"
assignee: "worker-task-219-0596f986"
completedAt: "2026-08-05T19:23:36Z"
completion:
  summary: "Hardened the local read-only web MVP with visible-tab revision polling, changed-view refresh, strict Host validation, restrictive security headers, request limits, no CORS, and safe error/content behavior."
  filesChanged: ["tandem/src/web.rs", "tandem/src/web/app.js", "tandem/src/web/ui.js", "tandem/src/web/app.css"]
  validation: "Integrated commit a2a0abb reviewed; 240 unit and 11 integration tests, formatting, strict all-feature Clippy, release build, JS syntax, diff checks, Chromium polling smoke, and parent live Host/header/method/no-CORS smoke passed."
  reviewer: "orchestrator"
---

## Description

Harden the local read-only web mode and keep views current without adding unnecessary infrastructure.

Acceptance criteria:
- Add simple 2–5 second revision polling while the page is visible and refetch changed canonical snapshots.
- Validate Host headers, serve no permissive CORS headers, and add restrictive CSP, frame denial, no-sniff, referrer, and no-store policies where appropriate.
- Escape project-controlled text and sanitize rendered Markdown. Do not load remote content automatically.
- Limit request sizes and avoid logging project bodies, tokens, secrets, or unrelated paths.
- Remain loopback-only and read-only. Do not add SSE, WebSockets, accounts, remote binding, or mutation endpoints.
- Add focused refresh and security regression tests.
