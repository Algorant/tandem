---
id: decision-13
uid: 0efc3655-b1ed-49c4-9935-901e57ef22a0
type: decision
title: "Tandem ships without a web viewer"
status: "proposed"
deciders: ["Algorant"]
references: ["task-117", "http://desktop.tail1cefc.ts.net:8228/session/jLMkuE8ueiY"]
createdAt: "2026-11-21T03:17:18Z"
updatedAt: "2026-11-21T03:17:18Z"
---

## Status
Accepted — Algorant, 2026-11-21, via Herdra m4f3c0814.

## Context
The local read-only browser and HTTP interface were distinct, but no recurring reader or client is known. The [research report](http://desktop.tail1cefc.ts.net:8228/session/jLMkuE8ueiY) compares the existing surfaces and upkeep.

## Decision
Remove `tandem web` now and ship without a web viewer or HTTP interface. No advance announcement. Board access is through the TUI and CLI; Herdra and Sideshow remain complementary surfaces for coordination and evidence. The public docs site stays.

## Consequences
Local browser browsing and HTTP clients cease to work; board records, protocol, sync and the normal single-binary packaging remain. Revisit only if a concrete browser use appears.
