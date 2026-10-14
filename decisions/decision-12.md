---
id: decision-12
uid: 6e6a6857-517b-4c41-b698-17a46432d436
type: decision
title: "Retired documentation URLs after the seven-page rewrite"
status: "proposed"
deciders: ["Algorant"]
references: ["task-76", "task-83"]
createdAt: "2026-10-14T04:39:28Z"
updatedAt: "2026-10-14T04:39:28Z"
---

## Status
Proposed. No final URL policy authorized; task-76's unpublished preview shows proposed cuts only.

## Context
Nineteen Markdown pages become seven canonical pages; old bookmarks/search results may remain. Existing /protocol/ points to /workspace/, now folded into Concepts. Maintainer guides leave public build/search. Decide with rendered pages in J0qH-JN7PqI before docs acceptance/publication.

## Options
1. Selected relevant redirects into seven pages (recommended), with retired maintainer paths returning404. No legacy content pages.
2. All retired routes404: simplest, breaks old links.
3. Redirect every retired route broadly: navigation preserved, some destinations misleading. Extra legacy landing pages add scope and are not recommended.

## Candidate map
/workspace/, /protocol/, /guides/upgrading-to-independent-sync/ → /concepts/#board-storage-and-sync
/web/ → /tui/#read-only-web-view
/extensions/, /skills/, /guides/agents-and-adapters/ → /agents/
/guides/decisions/, /guides/human-in-the-loop/, /guides/fully-agentic-workflow/ → /concepts/
/reference/ → /cli/
/guides/ → /concepts/ OR retire explicitly
/guides/docs-site/, /guides/theme-tester/, /packaging/aur-tandem-bin/ → no public replacement, proposed404

## Consequences
Record Algorant's choice before implementation/deployment. Use existing static-site configuration only; no server/proxy/network infrastructure. Seven canonical content pages remain. Preview404s do not imply acceptance.
