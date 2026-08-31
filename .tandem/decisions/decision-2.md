---
id: decision-2
type: decision
title: "Use Bun as the default JavaScript package manager and runner"
status: "accepted"
deciders: ["Algorant"]
tags: ["stack", "tooling", "docs", "rust", "typescript", "bun", "ci"]
createdAt: "2026-08-31T20:52:47Z"
decidedAt: "2026-07-04T00:00:00Z"
updatedAt: "2026-08-31T20:52:47Z"
---

## Status

Accepted.

## Context

Tandem uses JavaScript tooling for the Astro/Starlight docs site and Pi extension checks. Workflows historically defaulted to npm; the project direction is Bun-first with documented exceptions.

## Decision

Use Bun over npm for JavaScript package management, scripts, CI, and local automation by default. Do not keep npm-based workflows merely because they are familiar. npm remains allowed only as an explicit exception when Bun cannot satisfy a concrete requirement after reasonable attempts.

## Consequences

- npm lockfiles, workflows, and docs are removed or replaced once migration is validated.
- Exceptions document what was tried, why Bun failed, and what would allow revisiting the exception.
- Local recipes and CI stay aligned on one package manager.
