---
id: decision-6
type: decision
title: "Use one Git-merged event ledger with UUIDv7 identities"
status: "accepted"
date: "2026-07-15"
deciders: ["Algorant"]
context: "Tandem needs conflict-tolerant event sharing without external platform-specific state or durable files for every ephemeral worktree. Git's union merge driver can combine independently appended JSONL lines if events carry globally unique identities and readers ignore physical order."
consequences: ["Use one tracked `.tandem/events.jsonl` with `text eol=lf merge=union` configured in `.tandem/.gitattributes`.", "Require UUIDv7 `eventId` as canonical identity; retain actor attribution but remove `<actor>:<seq>` identity requirements.", "Store one random actor UUID per Git clone in clone-local Git config, shared by linked worktrees; use ignored workspace fallback outside Git.", "Read legacy event lines without rewriting them and detect conflicting payloads for duplicate new event IDs.", "Defer rotation, custom refs, external state, and database storage until demonstrated need."]
alternatives: ["Per-actor files keyed by worktree identity; rejected due durable file proliferation from ephemeral worktrees.", "External XDG/user-state storage; rejected as insufficiently Git-centric and unnecessarily platform-oriented.", "Per-actor Git refs or hidden branches; rejected initially due synchronization and plumbing complexity.", "Immediate coarse time segmentation; deferred until observed ledger growth warrants it."]
references: ["task-77", "task-119"]
tags: ["protocol", "events", "git", "cross-platform", "audit"]
createdAt: "2026-07-15T17:01:44Z"
updatedAt: "2026-07-15T17:01:44Z"
---

## Status

Accepted — approved by Algorant after resolving storage, identity, attribution, migration, legacy compatibility, and scope choices before implementation planning.

## Context

Tandem currently appends lifecycle records to `.tandem/events.jsonl`. Task-77 specified unimplemented per-actor files and `<actor>:<seq>` identities to avoid Git append conflicts. Task-119 later researched external user-state storage, but that recommendation was never accepted and does not match Tandem's Git-centric, cross-platform direction. Creating a durable file for every ephemeral worktree would also make storage topology reflect orchestration churn rather than meaningful project structure.

Git provides a built-in `union` text merge driver that retains lines appended independently on both sides, although merged line order is arbitrary. Tandem can therefore retain one simple tracked JSONL ledger if every new event has a globally unique identity and readers never treat physical line order as chronology.

## Decision

Tandem uses one tracked append-only ledger at `.tandem/events.jsonl`. `.tandem/.gitattributes` applies:

```gitattributes
events.jsonl text eol=lf merge=union
```

New events require a UUIDv7 `eventId`, timestamp, actor ID, event name, target document ID, and summary. `eventId` is canonical event identity; `<actor>:<seq>` is not used. Readers sort by timestamp and event ID, deduplicate identical records with the same event ID, and report the same event ID with differing payloads as corruption. Physical line order is not semantic.

The canonical actor ID is one random UUID per Git clone, stored in clone-local Git configuration as `tandem.actorId` and shared by that clone's linked worktrees. `actorName` is optional display metadata and never determines identity. Non-Git workspaces use an ignored `.tandem/actor-id` fallback; no XDG or platform-specific external state directory is required.

New workspaces receive the attributes rule during `tandem init`. Existing workspaces safely gain or augment `.tandem/.gitattributes` when Tandem first writes a new-format event. Tandem preserves unrelated rules and warns rather than overwriting a conflicting event rule.

Existing event lines without `eventId` remain readable without rewriting or migration. Only new records require UUIDv7 identity. The initial implementation keeps one file without rotation; yearly or other coarse segmentation is deferred until observed size or performance justifies it.

This decision supersedes the unimplemented task-77 per-actor-file and `<actor>:<seq>` design. Task-119 remains historical options research; its external user-state recommendation is rejected as the default. Custom refs, hidden branches, and external databases are not part of this implementation.

## Consequences

- Tandem remains Git-centric and portable across Linux, macOS, and Windows.
- Ordinary branches and worktrees can append independently; Git union-merges their event lines.
- Hundreds of ephemeral worktrees do not create hundreds of durable actor files.
- The ledger continues to dirty the checkout and grow over time, but growth and rotation remain evidence-driven rather than preemptively complex.
- Union merges may reorder records, so all readers and tests must use semantic event ordering.
- Semantic workflow conflicts remain separate from line-level merge success and must not be hidden by event merging.

## Alternatives considered

- Per-actor files keyed by worktree identity: rejected because ephemeral worktrees create durable file proliferation and cleanup questions.
- External XDG/user-state storage: rejected because it weakens Git portability and introduces platform-specific storage concerns.
- Per-actor Git refs or hidden branches: rejected for the initial design because synchronization, hosting, and plumbing add unnecessary complexity.
- Immediate yearly segmentation: deferred because a single ledger is simpler and current scale does not justify rotation.
