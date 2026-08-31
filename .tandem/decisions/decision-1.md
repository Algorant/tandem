---
id: decision-1
type: decision
title: "Use Astro Starlight and GitHub Pages for Tandem docs"
status: "accepted"
deciders: ["Algorant"]
tags: ["docs", "site", "deployment"]
createdAt: "2026-08-31T20:52:47Z"
decidedAt: "2026-08-31T00:00:00Z"
updatedAt: "2026-08-31T20:52:47Z"
---

## Status

Accepted.

## Context

The Tandem documentation site needed a maintained static stack with a docs source tree and deployment path.

## Decision

Adopt the Astro Starlight site under `site/` as the Tandem documentation site stack. Canonical Markdown lives under `docs/` and is synced into Starlight content; deployment uses GitHub Pages Actions. Revisit the stack only if Starlight cannot meet concrete design or UX needs.

## Consequences

- Docs content stays canonical Markdown under `docs/`.
- Site packaging and scripts align with the Bun decision.
- Deployment remains GitHub Pages via Actions.
