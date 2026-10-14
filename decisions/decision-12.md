---
id: decision-12
uid: 6e6a6857-517b-4c41-b698-17a46432d436
type: decision
title: "Retired documentation URLs after the seven-page rewrite"
status: "accepted"
deciders: ["Herdra"]
references: ["task-76", "task-83"]
createdAt: "2026-10-14T04:39:28Z"
updatedAt: "2026-10-14T04:45:44Z"
decidedAt: "2026-10-14T04:45:44Z"
---
## Status
Accepted for implementation by Herdra during task-76 rendered review. Algorant's final docs acceptance and publication remain pending; this is not authorization to deploy before that acceptance.

## Context
Nineteen public Markdown pages become seven canonical pages. Selected old links have meaningful replacements; maintainer content leaves the public site. Preview 404s were not treated as a decision.

## Decision
Use only existing site config redirects; no network/server/proxy changes or extra legacy content pages.

- /workspace/, /protocol/, /guides/upgrading-to-independent-sync/ → /concepts/#board-storage-and-sync
- /web/ → /tui/#read-only-web-view
- /extensions/, /skills/, /guides/agents-and-adapters/ → /agents/
- /guides/decisions/, /guides/human-in-the-loop/, /guides/fully-agentic-workflow/ → /concepts/
- /reference/ → /cli/
- /guides/ → /concepts/ (explicit Herdra choice)
- /guides/docs-site/, /guides/theme-tester/, /packaging/aur-tandem-bin/ → intentionally retired, return 404; maintainer sources remain outside public build/search

## Consequences
Seven canonical content pages remain; static redirect artifacts are aliases, not extra documentation pages. Verify generated targets/fragments and retired-route 404s in rendered preview. Present revised pages/map to Herdra before Algorant; hold integration/push/deploy and release 0.17.1 until accepted docs.