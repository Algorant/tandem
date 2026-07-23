---
title: Protocol
description: The Tandem local-first file format.
---
The Tandem protocol defines the local `.tandem/` data model used by the CLI, TUI, and integration adapters.

## Layout

```text
.tandem/
├── tandem.md        # workspace config and rules
├── board/           # active Markdown documents
├── logs/            # completed or canceled Markdown documents
├── events/          # per-actor append-only lifecycle events
│   └── <actor_id>.jsonl
└── events.jsonl     # legacy global event log; readable during transition
```

## Documents

V0 supports `task` and `decision` documents. Epics and Tasks use the global `task-N` namespace; only a Subtask directly beneath a Task uses the parent-derived `task-N-M` form.

Task documents use frontmatter for structured fields and Markdown for the human-readable body. Tools should preserve unknown fields and minimize rewrites. `tandem update --body` replaces an active Task's complete body exactly while preserving unrelated frontmatter.

## Epic convention

Epics are ordinary task documents with an optional classifier:

```yaml
id: task-10
type: task
kind: epic
title: Ship documentation refresh
state: in-progress
```

A direct Epic child is a global-ID Task linked through `parentId`:

```yaml
id: task-11
type: task
title: Rewrite Concepts page
state: todo
parentId: task-10
references:
  - decision-3
```

A direct Task child is a parent-derived leaf Subtask:

```yaml
id: task-11-1
type: task
title: Update hierarchy examples
state: todo
parentId: task-11
```

Tandem resolves the documents before deriving roles: `task-10` is an Epic, `task-11` is its Task with relationship `epic-task`, and `task-11-1` is that Task's Subtask with relationship `subtask`. A Task attached to a decision or custom document remains a global-ID Task with generic relationship `parent` and may own Subtasks.

`parentId` is strict hierarchy; `references` are loose related links. Epics are root-only, Subtasks cannot have children, and IDs are immutable. Role-changing or ID-invalidating reparenting is rejected, as are hierarchical direct Epic children and global-ID Subtasks. There is no legacy compatibility exception. Epic tasks are completed and archived with the normal task flow; v0 does not define `type: epic`, `epic-N` IDs, a separate ADR/epic record type, or special epic lifecycle behavior.

Only Tasks are initial delegation roots. One worker owns a delegated Task's direct Subtasks as its execution checklist; Epics and Subtasks are not independently delegated.

## Decisions

Decision documents are the ADR-compatible durable record type. Required fields are `id`, `type: decision`, and `title`; optional ADR-style metadata may include `status`, `date`, `deciders`, `tags`, `supersedes`, and `supersededBy`. Supersession links should also appear in `references` when current CLI/TUI relationship views should find them.

## Completion and cancellation

Successful completion and reasoned cancellation both archive the Task to `.tandem/logs/`, preserve its body/metadata, remove active `state`, and retain its ID. Archived Tasks require `completedAt` and `completion.summary`; the compatible optional `completion.outcome` is `completed` or `canceled`, with omission meaning completed for legacy Logs. Cancellation rejects active descendants and emits `task.canceled`; canceled work remains auditable but does not count as successful completion.

## Events

New event writes append to the current writer's `.tandem/events/<actor_id>.jsonl`; readers aggregate all per-actor logs plus any legacy `.tandem/events.jsonl`. Event records require `ts`, `event`, `id`, `summary`, canonical `actor`, and per-actor `seq`; the event identity is `<actor>:<seq>`. Optional `actorName` is display-only and never determines canonical identity or file ownership.

Per-actor logs avoid Git file-level append conflicts, but semantic conflicts between actors' task or review changes can still happen and should be surfaced rather than discarded.

## Validation rules

Built-in structural validation checks required fields, core relationships, derived hierarchy roles, and their required ID forms. Unresolved `parentId` or blockers, parented Epics, children beneath Subtasks, role/ID mismatches, and invalid reparenting are errors. Unresolved related references are warnings in v0.

## Design notes

- Protocol version starts at `0.1.0`.
- The canonical workflow field is `state`.
- The work agreement object is `accord`.
- Completion and cancellation are archive actions, not persistent Board states.
- Brainfile import or migration is not required for v0.

See `protocol/plan/spec.md` in the repository for the detailed draft while the public docs are still being expanded.
