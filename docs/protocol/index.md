---
title: Protocol
description: The Tandem local-first file format.
---

# Protocol

The Tandem protocol defines the local `.tandem/` data model used by the CLI, TUI, and integration adapters.

## Layout

```text
.tandem/
├── tandem.md        # workspace config and rules
├── board/           # active Markdown documents
├── logs/            # completed Markdown documents
├── events/          # per-actor append-only lifecycle events
│   └── <actor_id>.jsonl
└── events.jsonl     # legacy global event log; readable during transition
```

## Documents

V0 supports `task` and `decision` documents. Task IDs are sequential by default, such as `task-1`; subtasks use parent-based IDs such as `task-1-1`.

Task documents use frontmatter for structured fields and Markdown for the human-readable body. Tools should preserve unknown fields and minimize rewrites.

## Events

New event writes append to the current writer's `.tandem/events/<actor_id>.jsonl`; readers aggregate all per-actor logs plus any legacy `.tandem/events.jsonl`. Event records require `ts`, `event`, `id`, `summary`, canonical `actor`, and per-actor `seq`; the event identity is `<actor>:<seq>`. Optional `actorName` is display-only and never determines canonical identity or file ownership.

Per-actor logs avoid Git file-level append conflicts, but semantic conflicts between actors' task or review changes can still happen and should be surfaced rather than discarded.

## Validation rules

Built-in structural validation checks required fields and core relationships. Unresolved `parentId` or blockers are errors. Unresolved related references are warnings in v0.

## Design notes

- Protocol version starts at `0.1.0`.
- The canonical workflow field is `state`.
- The work agreement object is `accord`.
- Completion is an archive action, not a persistent `done` state.
- Brainfile import or migration is not required for v0.

See `protocol/plan/spec.md` in the repository for the detailed draft while the public docs are still being expanded.
