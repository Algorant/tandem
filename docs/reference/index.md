---
title: Reference
description: Compact reference material for Tandem.
---
Reference pages will hold stable command, field, and configuration details as Tandem matures.

Initial references to expand:

- CLI command reference.
- Task frontmatter fields.
- Epic convention (`type: task` plus `kind: epic`).
- Decision/ADR body template and metadata fields.
- Accord status reference.
- Workspace config and rules.
- Theme configuration keys.

## Papercut record

Papercuts live at `.tandem/papercuts/papercut-N.md`. Required fields are `id`, `title`, `status`, `createdAt`, and `updatedAt`. Status is `open` or `resolved`. Optional `references` and `tags` are arrays. A resolved record requires nested `resolution.note` and `resolution.resolvedAt`.

IDs are immutable and sequential. References are loose and unresolved targets warn. Papercuts are not general documents and do not participate in Board workflow, Logs, hierarchy, Accord, review, completion, or the TUI. Use `tandem papercut add|list|show|resolve`; global search reports their location as `papercuts`.
