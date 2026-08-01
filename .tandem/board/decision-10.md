---
id: decision-10
type: decision
title: "Make worktree-local event actor identity non-configurable"
status: "accepted"
date: "2026-07-30"
deciders: ["Algorant"]
context: "Decision-9 retained TANDEM_ACTOR_ID as a compatibility override, but no supported scripts or workflows use it and project integrations should not use it. A shared environment override could collapse independent worktrees onto one actor ledger and reintroduce sequence and merge conflicts."
consequences: ["Tandem always resolves actor identity from worktree-local `.tandem/actor-id`, creating a random UUID atomically when absent.", "TANDEM_ACTOR_ID has no effect and is not part of the supported protocol or CLI environment.", "Herdr, Worktrunk, Pi, pi-tandem, shell configuration, and other integrations cannot override or inject event actor identity.", "Existing tracked actor ledgers and persisted `.tandem/actor-id` files remain valid with no migration."]
alternatives: ["Retain TANDEM_ACTOR_ID with warnings; rejected because there is no current consumer and the override can defeat the isolation guarantee.", "Scope an override to tests only; rejected because tests can validate persisted identities without a production override."]
supersedes: ["decision-9"]
references: ["decision-9", "task-191"]
tags: ["protocol", "events", "identity", "worktrees", "safety"]
createdAt: "2026-07-31T02:38:03Z"
updatedAt: "2026-07-31T02:38:03Z"
---

## Status

Accepted. This decision narrows and supersedes decision-9 only where decision-9 retained an explicit actor environment override.

## Context

The worktree-local identity design exists to guarantee that independent writable checkouts use separate actor ledgers. `TANDEM_ACTOR_ID` could bypass that boundary. No supported script, workflow, or integration requires the override, and retaining an unused compatibility path would add a configuration footgun.

## Decision

Event actor identity is non-configurable. Tandem always reuses the canonical random UUID in `.tandem/actor-id` or atomically creates it on the first mutation. Environment variables, agent names, machine names, user names, and integration configuration cannot override this identity.

Tandem remains the sole owner of identity generation, persistence, validation, and event writing. Herdr, Worktrunk, Pi, `pi-tandem`, Workers, Reviewers, and Subagents remain identity-unaware.

## Consequences

- One independent writable checkout or linked worktree has exactly one local actor identity.
- A global environment variable cannot collapse separate worktrees onto one ledger.
- Existing per-actor ledgers, sequence identities, legacy reads, and persisted actor UUIDs remain unchanged.
- No migration or history rewrite is required.
