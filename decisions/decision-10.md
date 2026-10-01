---
id: decision-10
uid: af79815f-9271-4d06-a106-d4b0996e71e2
type: decision
title: "Retire the pi-tandem adapter from the Tandem repository"
status: "proposed"
deciders: ["Algorant"]
references: ["task-58", "task-55"]
createdAt: "2026-10-01T23:35:47Z"
updatedAt: "2026-10-01T23:35:47Z"
---

## Status
Accepted (2026-10-01, Algorant, relayed by the ~/.pi orchestrator).

## Context
`extensions/pi-tandem` was a thin Pi adapter over the `tandem` CLI. Its `tandem_papercut` tool classified papercuts with `--tag papercut`, which task-55 made obsolete when papercut became a first-class `kind` (protocol 0.5.0, no tag fallback). The ~/.pi configuration now maintains its own private Tandem tools and no longer calls this adapter.

## Decision
Retire `extensions/pi-tandem` instead of updating it. Remove it from this repository. Docs that described it (docs/extensions/index.md and other references) point to the `tandem` CLI, with its `--json` output, as the integration surface instead.

## Consequences
- Tandem ships no Pi adapter. Integrations call the CLI directly.
- ~/.pi keeps its adapter privately, outside this repository.
- The `extensions/` area remains the home for any future integration. Rule never-2 continues to protect adapter code that is added later.

## Supersession
None.
