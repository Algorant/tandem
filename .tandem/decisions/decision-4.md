---
id: decision-4
type: decision
title: "Curate concise public release notes without installation guidance"
status: "accepted"
deciders: ["Algorant"]
tags: ["docs", "release", "decision", "guidance"]
createdAt: "2026-08-31T20:52:47Z"
decidedAt: "2026-08-31T00:00:00Z"
updatedAt: "2026-08-31T20:52:47Z"
---

## Status

Accepted.

## Context

Public release notes serve readers deciding what changed in a version; they should not duplicate the reusable release procedure, installation docs, or a raw commit inventory. Prior research recommended curated per-release notes separated from the operational checklist. External conventions converge on a small conditional vocabulary.

## Decision

Public release notes are concise, version-specific, manually curated by user impact, built from a conditional template: version/title plus a one- or two-sentence summary; optional Highlights, Breaking changes, Features, Improvements, and Security/Deprecations/Compatibility/Known issues sections when applicable; and a dedicated Bug fixes section whenever user-visible defect corrections ship. Never emit empty sections. Product surfaces may group notes when a large release benefits, but are not required top-level headings. Public notes never contain installation commands or guidance; migration/compatibility actions belong under Breaking changes or Compatibility. Do not mention rejected, shelved, or not-shipped work; generated output is drafting input only, never published unchanged.

## Consequences

- Release authors maintain a concise curated notes file per release; procedural detail stays in the release checklist.
- Fixes are clearly grouped and discoverable.
- Installation material remains out of public release bodies.
