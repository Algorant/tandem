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
└── events.jsonl     # append-only lifecycle events
```

## Documents

V0 supports `task` and `decision` documents. Task IDs are sequential by default, such as `task-1`; subtasks use parent-based IDs such as `task-1-1`.

Task documents use frontmatter for structured fields and Markdown for the human-readable body. Tools should preserve unknown fields and minimize rewrites.

## Validation rules

Built-in structural validation checks required fields and core relationships. Unresolved `parentId` or blockers are errors. Unresolved related references are warnings in v0.

## Design notes

- Protocol version starts at `0.1.0`.
- The canonical workflow field is `state`.
- The work agreement object is `accord`.
- Completion is an archive action, not a persistent `done` state.
- Brainfile import or migration is not required for v0.

See `protocol/plan/spec.md` in the repository for the detailed draft while the public docs are still being expanded.
