---
id: task-191
type: task
title: "Persist automatic worktree-local event actor identity"
priority: "high"
references: ["decision-9", "decision-6", "task-133", "task-135", "task-136", "task-137", "task-138"]
relatedFiles: ["protocol/plan/spec.md", "protocol/README.md", "tandem/src/project/events.rs", "tandem/src/project/mod.rs", "tandem/src/protocol/event.rs", "tandem/tests/cli_behavior.rs", "extensions/pi-tandem/README.md", "AGENTS.md", "docs/protocol/index.md"]
tags: ["protocol", "events", "identity", "git", "worktrees"]
createdAt: "2026-07-30T13:05:22Z"
updatedAt: "2026-07-30T13:22:30Z"
accord:
  status: "accepted"
  assignee: "worker-task-191-6c244763"
  claimedAt: "2026-07-30T13:11:08Z"
  deliveredAt: "2026-07-30T13:22:18Z"
  deliverables: ["Integrated squash commit 07aca81 on main", "Persistent `.tandem/actor-id` resolution and atomic creation", "Git clone and linked-worktree ignore handling", "Protocol, repository, docs, and pi-tandem boundary updates", "Unit and process-level compatibility tests"]
  validation:
    commands: ["cargo test --manifest-path tandem/Cargo.toml: 210 unit tests and 11 CLI tests passed after integration", "cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings: passed after integration", "Worker validation: rustfmt, TypeScript checks, and all three pi-tandem smoke suites passed"]
  summary: "Implemented and integrated persistent checkout/worktree-local actor UUIDs with override precedence, atomic concurrent creation, Git exclude handling, non-Git fallback without a Git executable, strict diagnostics, documentation, and compatibility tests."
  filesChanged: ["AGENTS.md", "docs/protocol/index.md", "extensions/pi-tandem/README.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/src/project/events.rs", "tandem/tests/cli_behavior.rs"]
  reviewer: "orchestrator"
  note: "Accepted after code review, one rework cycle for non-Git operation without a Git executable, successful Worktrunk integration, and independent post-integration Rust tests and strict Clippy."
  updatedAt: "2026-07-30T13:22:22Z"
assignee: "worker-task-191-6c244763"
completedAt: "2026-07-30T13:22:30Z"
completion:
  summary: "Implemented decision-9 with persistent worktree-local actor UUIDs, explicit override precedence, atomic concurrent identity creation, Git-local ignore handling, non-Git compatibility, preserved per-actor event semantics, updated integration guidance, and passing full validation."
  filesChanged: ["AGENTS.md", "docs/protocol/index.md", "extensions/pi-tandem/README.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/src/project/events.rs", "tandem/tests/cli_behavior.rs"]
  validation: "Integrated as 07aca81. Post-integration `cargo test --manifest-path tandem/Cargo.toml` passed 210 unit and 11 CLI tests. Post-integration strict Clippy passed. Worker rustfmt, TypeScript checks, and all pi-tandem smoke suites passed."
  reviewer: "orchestrator"
---
## Description

Implement decision-9 without changing the released per-actor event storage or event identity model. Replace process-local fallback identities with one persistent random actor UUID per independent checkout or linked worktree.

## Required behavior

- Resolve event actor identity in this order: explicit filename-safe `TANDEM_ACTOR_ID`, existing worktree-local `.tandem/actor-id`, then a newly generated and atomically persisted random UUID.
- Treat the actor as an independent writable checkout/worktree identity, not a person, machine, agent, clone-wide identity, or repository-wide identity.
- Keep `.tandem/events/<actor-id>.jsonl`, `<actor>:<seq>`, legacy `.tandem/events.jsonl` reads, current locking, sequence validation, and corruption diagnostics unchanged.
- Ensure `.tandem/actor-id` is ignored in Git projects without dirtying tracked project policy. Resolve Git paths correctly for normal clones and linked worktrees; do not assume `.git` is a directory.
- For non-Git Tandem workspaces, persist `.tandem/actor-id` locally and document its local-runtime role.
- Handle concurrent first mutation safely so competing processes converge on one persisted identity rather than creating or overwriting different IDs.
- Fail clearly for malformed, unsafe, unreadable, or unwritable identity state; do not silently fall back to a new process identity.
- Keep `TANDEM_ACTOR_ID` as an intentional override, and document that one shared global value must not be injected across independent Herdr/Worktrunk worktrees.

## Integration boundary

- Tandem owns identity generation, persistence, validation, and event writing.
- Herdr and Worktrunk need no identity lifecycle implementation: new worktrees generate an identity, retained/recovered Workers reuse it, and cleanup removes the ignored identity with the worktree while committed actor ledgers remain history.
- `pi-tandem`, Pi Workers, Reviewers, and Subagents must not generate, copy, parse, or automatically pass actor IDs.
- Update repository and integration guidance only where needed to make this boundary explicit.

## Validation

- Test identity reuse in one checkout and distinct identities in independent clones and linked worktrees.
- Test `TANDEM_ACTOR_ID` precedence and invalid override diagnostics.
- Test concurrent first use and concurrent same-worktree appends.
- Verify the identity file is ignored while the actor ledger remains tracked.
- Verify retained/recovered-worktree continuity and new-worktree isolation through a focused delegation/worktree smoke where practical.
- Verify existing per-actor and legacy event history remains readable with no rewrite or migration.
- Run Rust formatting, focused tests, full tests, strict Clippy, and relevant `pi-tandem` smoke/documentation checks.

## Explicit non-goals

- Single union-merged event ledger.
- UUIDv7 event identity.
- Event `.gitattributes` union rules.
- Event-history migration or rewriting.
- External XDG state, Git refs, hidden branches, databases, distributed locking, rotation, or pluggable storage backends.
- Solving semantic conflicts in Task, Decision, Rule, accord, review, completion, or sequential-ID documents.
