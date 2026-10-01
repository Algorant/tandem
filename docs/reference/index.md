---
title: Reference
description: Compact reference material for Tandem.
---
Reference pages will hold stable command, field, and configuration details as Tandem matures.

Initial references to expand:

- CLI command reference.
- Task frontmatter fields.
- Task kinds (`type: task` plus `kind: epic|research|papercut`).
- Decision/ADR body template and metadata fields.
- Accord status reference.
- Workspace config and rules.
- Theme configuration keys.

## Task kinds

`kind` is optional on a Task and is one of `epic`, `research`, or `papercut`. A Task without `kind` is standard. Tags are topical only and are never read as a kind.

| Kind | Minimum at `add` | Default | Placement |
| --- | --- | --- | --- |
| none | title + acceptance | none | root, Epic child, or Subtask |
| `epic` | title + acceptance | none | root only |
| `research` | title + acceptance | none | anywhere |
| `papercut` | title only | `priority: low` | root or direct child of an Epic; never a Subtask |

Only a papercut may have no acceptance. Creating, reparenting, or re-kinding a papercut into a Subtask fails validation. Use `tandem list --kind <kind>`, `tandem search <query> --kind <kind>`, and `tandem add task --kind <kind>`. `tandem migrate` converts old `research`/`papercut` tags on active Board Tasks once; archived Logs keep their legacy tags.
