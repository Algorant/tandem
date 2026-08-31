---
id: task-136
type: task
title: "Implement UUIDv7 event writes and Git-local actor identity"
priority: "high"
blockers: ["task-135"]
references: ["decision-6", "decision-7"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/rules.rs"]
tags: ["rust", "cli", "tui", "events", "git", "uuid"]
createdAt: "2026-07-15T19:43:30Z"
updatedAt: "2026-07-30T13:05:49Z"
parentId: "task-133"
completedAt: "2026-07-30T13:05:49Z"
completion:
  outcome: "canceled"
  summary: "Canceled: Superseded by decision-9 after the per-actor event ledger shipped in d974133. UUIDv7 union-ledger writes and Git attributes are not required for the narrower identity fix."
---

## Description

This is a direct Task of Epic task-133. Implement decision-6's new event writer, actor resolver, and Git attributes initialization in the Rust CLI/TUI mutation paths.

Acceptance criteria:
- Generate a UUIDv7 `eventId` for every new event and emit the required new envelope without `seq`.
- Resolve/create one random actor UUID through clone-local Git config `tandem.actorId`, shared by linked worktrees; never use actorName as identity.
- Outside Git, resolve/create an ignored `.tandem/actor-id` fallback without OS-specific external paths.
- `tandem init` creates `.tandem/.gitattributes` with `events.jsonl text eol=lf merge=union` and initializes the ledger safely.
- Existing workspaces safely gain or augment the rule on first new-format write; preserve unrelated rules and warn/refuse rather than overwrite a conflicting event rule.
- All CLI and TUI mutation paths use one shared writer implementation and append only complete single-line JSON records.
- Preserve current mutation/event failure diagnostics and add focused tests for Git clones, linked worktrees, non-Git fallback, existing attributes, conflicts, UUID uniqueness/version, and cross-platform path/line-ending behavior.
- Keep one ledger with no rotation or custom-ref/backend abstraction.
