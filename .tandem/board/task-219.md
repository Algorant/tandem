---
id: task-219
type: task
title: "Add web refresh and local security safeguards"
state: todo
priority: "high"
parentId: "task-216"
blockers: ["task-217", "task-218"]
references: ["task-121"]
relatedFiles: ["tandem/src"]
tags: ["ui", "web", "security", "refresh"]
createdAt: "2026-08-05T18:46:23Z"
updatedAt: "2026-08-05T18:46:23Z"
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
