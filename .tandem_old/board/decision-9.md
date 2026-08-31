---
id: decision-9
type: decision
title: "Retain per-actor event ledgers with worktree-local writer identity"
status: "accepted"
date: "2026-07-28"
deciders: ["Algorant"]
context: "The per-actor event ledger shipped after decision-6 and now provides conflict isolation, legacy aggregation, file locking, sequence validation, and corruption diagnostics. Replacing it with a single UUIDv7 union-merged ledger would add a second storage transition without demonstrated need. The actual observed problem is process-local fallback identity, which creates many small ledgers and does not preserve writer continuity."
consequences: ["Keep tracked `.tandem/events/<actor-id>.jsonl` ledgers and `<actor>:<seq>` event identities.", "Replace process-local fallback identity with one random persistent identity per independent checkout or linked worktree.", "Keep `TANDEM_ACTOR_ID` as an explicit override, but integrations must not inject one shared value across independent worktrees.", "Store automatic identity in ignored `.tandem/actor-id`; Tandem owns resolution and integrations remain identity-unaware.", "Keep legacy `.tandem/events.jsonl` reads and do not migrate or rewrite event history.", "Do not implement the single union-merged ledger, UUIDv7 event identities, or `.gitattributes` union rule without new evidence."]
alternatives: ["Implement decision-6's single union-merged UUIDv7 ledger; rejected because the released per-actor baseline already solves the event append hotspot and another migration is not justified.", "Keep process-local fallback identities; rejected because they create excessive one-process ledgers and lose writer continuity.", "Use one machine- or clone-wide identity; rejected because concurrent linked worktrees could share a ledger and allocate conflicting sequences.", "Make Herdr, Worktrunk, or pi-tandem assign identities; rejected because Tandem should own protocol identity and persistence."]
supersedes: ["decision-6"]
references: ["decision-6", "task-133", "task-135", "task-136", "task-137", "task-138"]
tags: ["protocol", "events", "identity", "git", "worktrees"]
createdAt: "2026-07-30T13:04:54Z"
updatedAt: "2026-07-30T13:04:54Z"
---

## Status

Accepted. This decision supersedes decision-6 after the project baseline changed.

## Context

Decision-6 and Epic task-133 assumed the per-actor event design was not implemented. Commit `d974133` subsequently shipped tracked per-actor ledgers, followed by hardening for corrupt ledgers and actor identity parsing. The released design now isolates independent event writers, aggregates legacy and per-actor sources, locks same-ledger appends, validates per-actor sequences, and reports corruption.

The observed deficiency is narrower: when `TANDEM_ACTOR_ID` is absent, Tandem generates a process-local fallback. Repeated CLI invocations therefore create many small event files and do not retain a stable writer identity.

## Decision

Retain tracked `.tandem/events/<actor-id>.jsonl` ledgers and canonical `<actor>:<seq>` event identity. Keep `.tandem/events.jsonl` as readable legacy transition history.

Define an actor as one independent writable checkout or linked worktree, not a person, machine, agent, or whole Git repository. On the first mutation, Tandem resolves identity in this order:

1. An explicit filename-safe `TANDEM_ACTOR_ID` override.
2. An existing worktree-local `.tandem/actor-id`.
3. A newly generated random UUID persisted atomically to `.tandem/actor-id`.

The identity file is local runtime state and must be ignored by Git. The per-actor ledger remains tracked audit history. Processes in one worktree share the identity and rely on existing ledger locking. Independent clones and linked worktrees receive different automatic identities.

Tandem owns actor resolution. Herdr, Worktrunk, Pi Workers, Reviewers, Subagents, and the `pi-tandem` adapter must not generate, copy, parse, or globally inject actor identity. New worktrees naturally generate a new identity; retained and recovered Workers continue using the identity already present in their worktree.

Do not implement a single union-merged ledger, UUIDv7 event identities, or an event-specific `.gitattributes` rule without new evidence that the retained model is insufficient.

## Consequences

- Existing event history and protocol records require no migration or rewrite.
- Normal CLI use creates approximately one actor ledger per writable checkout/worktree rather than one per process.
- Independent Worker branches usually modify different event paths and merge normally.
- Ephemeral worktrees still leave their tracked audit ledger in project history after integration, while their ignored identity file disappears during cleanup.
- A globally shared `TANDEM_ACTOR_ID` can defeat worktree isolation and should not be configured by orchestration integrations.
- Event storage does not prevent semantic conflicts in Task, Decision, Rule, accord, review, completion, or sequential-ID documents.
