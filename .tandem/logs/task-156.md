---
id: task-156
type: task
title: "Extract TandemProject writes, locking, conflicts, and event files"
priority: "high"
parentId: "task-146"
blockers: ["task-155"]
references: ["decision-7"]
relatedFiles: ["plan/refactor_spec.md", "tandem/src/main.rs", "tandem/src/project/frontmatter.rs", "tandem/src/project/write.rs", "tandem/src/project/events.rs", "protocol/plan/spec.md"]
tags: ["protocol", "rust", "architecture", "refactor"]
createdAt: "2026-07-22T20:41:52Z"
updatedAt: "2026-07-27T03:46:28Z"
accord:
  status: "accepted"
  assignee: "worker-task-156-e932e90f"
  claimedAt: "2026-07-26T23:15:30Z"
  deliveredAt: "2026-07-27T03:46:15Z"
  deliverables: ["tandem/src/project/frontmatter.rs", "tandem/src/project/write.rs", "tandem/src/project/events.rs", "TandemProject mutation and event-read boundaries", "CLI/TUI migration to project write primitives"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml -- --check", "cargo test --manifest-path tandem/Cargo.toml --no-fail-fast: 194 unit + 4 executable tests passed", "cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings", "Dependency/write audit: no production durable project writes outside project and no protocol->project dependency"]
  summary: "Extracted project-owned frontmatter patches, locking/snapshots/conflicts, atomic writes and archives, initialization, and canonical per-actor event ledgers with legacy aggregation."
  filesChanged: ["tandem/src/project/events.rs", "tandem/src/project/frontmatter.rs", "tandem/src/project/write.rs", "tandem/src/project/mod.rs", "tandem/src/protocol/event.rs", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/logs.rs", "tandem/tests/cli_behavior.rs"]
  reviewer: "orchestrator"
  note: "Accepted after multiple focused rework cycles and independent validation. Canonical per-actor event identity/sequence, tolerant corruption diagnostics, escape-aware parsing, writer-unique fallback identity, atomic writes/archive rollback, full tests, formatting, strict Clippy, and dependency/write audits satisfy the task."
  updatedAt: "2026-07-27T03:46:21Z"
assignee: "worker-task-156-e932e90f"
completedAt: "2026-07-27T03:46:28Z"
completion:
  summary: "Completed the TandemProject write boundary, including safe atomic mutations and canonical per-actor event ledgers with legacy-compatible aggregation."
  filesChanged: ["tandem/src/project/events.rs", "tandem/src/project/frontmatter.rs", "tandem/src/project/write.rs", "tandem/src/project/mod.rs", "tandem/src/protocol/event.rs", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/logs.rs", "tandem/tests/cli_behavior.rs"]
  validation: "194 unit tests and 4 real-command tests passed; formatting and strict all-target Clippy passed; write/dependency audits found no direct durable interface/protocol writes."
  reviewer: "orchestrator"
---

## Description

## Objective

Complete the concrete project boundary for safe minimal filesystem mutations requested by application operations.

## Scope

- Extract raw frontmatter minimal-patch generation, snapshots, conflict detection, hierarchy lock lifetime, sequential file creation, atomic writes/replacements, Board-to-Logs moves, and per-actor event JSONL operations.
- Make `project::TandemProject` the concrete entry point for these operations.
- Consume protocol values/validated requests without inferring roles, transitions, status vocabularies, or diagnostic severity.
- Preserve unknown frontmatter/body content, IDs, references, event sequence/identity, and file atomicity.
- Retain one filesystem implementation; do not introduce repository, storage, transaction, or filesystem traits.

## Acceptance criteria

- All concrete `.tandem` mutation primitives live under `project`; protocol and interfaces perform no direct durable writes.
- Existing concurrency/allocation locking, stale-snapshot conflict behavior, event append, completion archive, and raw-preservation tests remain green.
- Failure paths leave no partial files, duplicate IDs, lost bodies, or malformed event lines.
- Formatting, full tests, concurrency tests, real-command tests, strict Clippy, and dependency/visibility review pass.
- Temporary lint expectations assigned to project writes are removed.
- No app orchestration/interface redesign, protocol change, release, or push occurs.

Creating this Task does not authorize starting it.
