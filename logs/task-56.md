---
id: task-56
uid: 1bd31435-ea2b-4099-ba34-a558f0e8d3f3
type: task
title: "Add typed links between records: relates-to, duplicates, fixed-by, supersedes"
priority: "low"
effort: "medium"
references: ["task-22", "task-55"]
tags: ["protocol", "relationships"]
accord:
  status: "accepted"
  acceptance: ["The protocol defines typed links between records (at least `relates-to`, `duplicates`, `fixed-by`/`fixes`, `supersedes`) and documents their storage shape and how they relate to the existing `references`.", "The CLI can add, remove, show, and filter typed links, and `complete` can record that a Task was resolved as fixed by another record.", "Tests cover link validation (unknown targets, self-links, archived targets) and the fixed-by completion path."]
  claimedAt: "2026-10-01T22:43:32Z"
  deliveredAt: "2026-10-01T23:05:19Z"
  validation: ["$ just dev-check"]
  summary: "Rebased onto main d60f525 (protocol 0.5.0) as a single commit, 24e4184, and `just dev-check` passes. The CLI contract in my previous ready report is unchanged. `links` is documented as part of 0.5.0 and needs no migration step.\n\nREBASE CONFLICT\n- The only conflict was `protocol/README.md`, in the CLI-contract paragraph. I kept main's 0.5.0 wording (including `--kind epic|research|papercut` and \"`migrate` converts 0.3.0 and 0.4.0 boards once\") and added the `link add|remove`, `list --link/--linked-to`, and `complete --fixed-by` sentence.\n- The leaf count stays 30 (28 plus `link add|remove`).\n- The compatibility sentence now reads: \"older versions reject the new protocol version, the `research` and `papercut` kinds, and typed `links`.\"\n- `landing.rs` auto-merged. It keeps main's text and gains one `link add|remove` line. Its existing \"0.4.0\" header text came from main and is unchanged.\n- `cli/model.rs` and `cli/commands.rs` auto-merged. `list --help` shows `--kind`, `--link` and `--linked-to` together.\n- My unit-test workspaces now use `protocolVersion: 0.5.0`."
  evidence: ["Protocol defines typed links, their storage shape and their relation to `references`, as part of 0.5.0: protocol/links.md says 'Protocol 0.5.0 adds typed links' and that no migration step is needed. protocol/README.md points to it and notes that older versions reject `links`.", "CLI can add, remove, show, filter links and complete --fixed-by: tests/link_behavior.rs passes post-rebase (5 tests) alongside the unit tests in app/links.rs and protocol/links.rs.", "Tests cover link validation (unknown targets, self-links, archived targets) and the fixed-by completion path: The unit and integration tests covering these cases pass on the rebased tree (339 unit tests plus 5 `link_behavior` tests, all passing)."]
  filesChanged: ["protocol/links.md", "protocol/README.md", "tandem/src/protocol/links.rs", "tandem/src/protocol/mod.rs", "tandem/src/protocol/event.rs", "tandem/src/protocol/merge.rs", "tandem/src/project/frontmatter.rs", "tandem/src/app/links.rs", "tandem/src/app/mod.rs", "tandem/src/app/tasks.rs", "tandem/src/app/queries.rs", "tandem/src/app/dto.rs", "tandem/src/cli/model.rs", "tandem/src/cli/commands.rs", "tandem/src/cli/landing.rs", "tandem/tests/link_behavior.rs"]
  updatedAt: "2026-10-01T23:05:20Z"
createdAt: "2026-10-01T22:21:03Z"
updatedAt: "2026-10-01T23:05:20Z"
relatedFiles: ["protocol/README.md", "tandem/src/protocol/document.rs", "tandem/src/app", "tandem/src/cli"]
assignee: "worker-task-56-86f3e9df"
archivedAt: "2026-10-01T23:05:20Z"
resolution:
  outcome: "completed"
---

## Description

## Why
`references` is untyped today. Pi (~/.pi) wants papercuts to stay standalone while connecting to the work they relate to. The main case: a papercut that another Task's work already fixed can be closed as `fixed-by` that Task, citing it, instead of with a manual note.

Independent of task-55. Discussion and mockups: http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8
