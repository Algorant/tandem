---
id: task-133
type: task
kind: "epic"
title: "Implement the Git-native union-merged event ledger"
priority: "high"
references: ["decision-6", "task-77", "task-119", "decision-7", "task-134"]
relatedFiles: ["protocol/plan/spec.md", "protocol/README.md", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/logs.rs", "tandem/plan/spec.md", "docs/protocol/index.md", "docs/concepts/index.md"]
tags: ["epic", "protocol", "events", "git", "cli", "tui", "cross-platform"]
createdAt: "2026-07-15T17:01:54Z"
updatedAt: "2026-07-30T13:05:55Z"
blockers: ["task-134"]
completedAt: "2026-07-30T13:05:55Z"
completion:
  outcome: "canceled"
  summary: "Canceled: Superseded by decision-9 after the per-actor event ledger shipped and was hardened. The Epic's assumed unimplemented baseline is stale; the narrower worktree-local actor identity improvement is now tracked by task-191."
---

## Description

Implement accepted decision-6 end to end across protocol, CLI, TUI readers, initialization, compatibility, tests, and public documentation.

Outcome:
- Tandem writes one tracked `.tandem/events.jsonl` ledger whose new records use UUIDv7 `eventId` identity.
- `.tandem/.gitattributes` configures `events.jsonl text eol=lf merge=union` for cross-platform conflict-tolerant appends.
- Actor attribution uses one random clone-local Git actor UUID shared by linked worktrees, with an ignored workspace fallback outside Git.
- Readers support mixed legacy/new records, semantic ordering, new-ID deduplication, and conflicting-payload diagnostics.
- The unimplemented per-actor-file and `<actor>:<seq>` design is removed from current specifications and documentation.

This Epic contains globally numbered Tasks task-135 through task-138 linked through canonical `parentId`; only a Task may own parent-derived Subtasks. Keep this Epic open until all implementation, compatibility, documentation, and validation Tasks are accepted and completed.
