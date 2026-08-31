---
id: decision-7
type: decision
title: "Keep worktree-local actor identity non-configurable"
status: "accepted"
deciders: ["Algorant"]
tags: ["protocol", "events", "identity", "worktrees", "safety"]
createdAt: "2026-08-31T20:52:47Z"
decidedAt: "2026-07-15T00:00:00Z"
updatedAt: "2026-08-31T20:52:47Z"
---

## Status

Accepted. Narrows and supersedes the earlier actor-override design, which is not retained in this workspace (see `.tandem_old`).

## Context

Worktree-local event identity guarantees that independent writable checkouts use separate actor ledgers. An environment override could bypass that boundary; no supported script, workflow, or integration required it.

## Decision

Event actor identity is non-configurable. Tandem always reuses the canonical random UUID in `.tandem/actor-id` or atomically creates it on the first mutation. Environment variables, agent names, machine names, user names, and integration configuration cannot override this identity. Tandem remains the sole owner of identity generation, persistence, validation, and event writing; Herdr, Worktrunk, Pi, and adapters remain identity-unaware. The file stays ignored and is never committed.

## Consequences

- One independent writable checkout or linked worktree has exactly one local actor identity.
- A global environment variable cannot collapse separate worktrees onto one ledger.
- Persisted actor UUIDs and per-actor ledgers remain valid with no migration.
