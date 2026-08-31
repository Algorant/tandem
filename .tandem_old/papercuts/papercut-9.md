---
id: papercut-9
title: "tandem update cannot remove entries from list fields"
status: open
createdAt: "2026-08-25T12:42:45Z"
updatedAt: "2026-08-25T12:42:45Z"
references: ["task-242"]
tags: ["cli", "update"]
---
`tandem update <id> --tag/--related-file/--reference` is additive only. Passing a narrower list unions with the existing value instead of replacing it, and passing a list that is a strict subset reports "No changes".

Hit while narrowing task-242's scope. `relatedFiles` still lists `scripts/benchmark_tui_idle.py` and `tandem/README.md`, and `tags` still contains `tui`, all of which are now explicitly out of scope. The only way to correct them is editing frontmatter by hand, which the project rules forbid.

`--body` already supports full replacement including clearing via an empty string. List fields have no equivalent.
