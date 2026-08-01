---
id: task-158
type: task
title: "Unify Rules and Decision application operations"
priority: "medium"
parentId: "task-146"
blockers: ["task-157"]
references: ["task-145"]
relatedFiles: ["plan/refactor_spec.md", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/decisions.rs", "tandem/src/app/rules.rs", "tandem/src/app/decisions.rs"]
tags: ["protocol", "cli", "tui", "rust"]
createdAt: "2026-07-22T20:42:14Z"
updatedAt: "2026-07-28T02:36:58Z"
accord:
  status: "accepted"
  assignee: "worker-task-158-7dcc0456"
  claimedAt: "2026-07-28T02:21:38Z"
  deliveredAt: "2026-07-28T02:36:17Z"
  deliverables: ["Shared app::rules add/edit/delete operations", "Shared app::decisions creation and diagnostics", "Protocol-owned Rules/Decision canonical values", "Project-owned byte-preserving Rules persistence", "CLI/TUI mutation parity coverage"]
  validation:
    commands: ["204 unit tests passed", "5 real-command tests passed", "formatting and strict Clippy passed", "root-import/reverse-dependency audits passed", "CLI whitespace preservation and TUI trim/clear tests passed"]
  summary: "Unified Rules and Decision application mutations with corrected protocol/project/app ownership and preserved CLI/TUI interface semantics."
  filesChanged: ["tandem/src/app/decisions.rs", "tandem/src/app/mod.rs", "tandem/src/app/rules.rs", "tandem/src/app/support.rs", "tandem/src/project/mod.rs", "tandem/src/project/rules.rs", "tandem/src/protocol/config.rs", "tandem/src/main.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/rules.rs", "tandem/tests/cli_behavior.rs"]
  reviewer: "orchestrator"
  note: "Accepted after rework corrected app-layer ownership and CLI/TUI parity; independent full validation and architectural audits passed."
  updatedAt: "2026-07-28T02:36:23Z"
assignee: "worker-task-158-7dcc0456"
completedAt: "2026-07-28T02:36:58Z"
completion:
  summary: "Unified Rules and Decision application operations with corrected protocol/project/app ownership and preserved CLI/TUI semantics. Integrated as e6bf181 into refactor/protocol-architecture and revalidated on target with 204 unit tests, 5 real-command tests, formatting, strict Clippy, diff, dependency, and parity audits."
  filesChanged: ["tandem/src/app/decisions.rs", "tandem/src/app/mod.rs", "tandem/src/app/rules.rs", "tandem/src/app/support.rs", "tandem/src/project/mod.rs", "tandem/src/project/rules.rs", "tandem/src/protocol/config.rs", "tandem/src/main.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/rules.rs", "tandem/tests/cli_behavior.rs"]
  validation: "Target: cargo fmt --all --manifest-path tandem/Cargo.toml -- --check; cargo test --manifest-path tandem/Cargo.toml --no-fail-fast (204+5 passed); cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings; git diff --check; app concern/root-import, reverse-dependency, TUI duplication, CLI byte-preservation and TUI normalization audits"
  reviewer: "orchestrator"
---

## Description

## Objective

Complete the shared application boundary for Rules mutations and Decision creation while preserving their existing interface behavior.

## Scope

- Move Rules add/edit/delete and Decision creation/diagnostic orchestration into cohesive `app::rules` and `app::decisions` operations.
- Compose canonical protocol semantics with concrete TandemProject operations and return typed outcomes/warnings.
- Switch CLI and TUI feature adapters to the same app operations and remove duplicated durable mutation logic.
- Keep Rules/Decisions TUI feature modules vertically cohesive; do not split their transient state/input/rendering merely for symmetry.
- Preserve ADR-compatible metadata, unknown fields/bodies, rule IDs/categories/sources, references, events, output, selection, and reload behavior.

## Acceptance criteria

- CLI and TUI durable Rules/Decision mutations use shared app operations.
- App code contains no printing, Ratatui rendering, process parsing, or direct protocol duplication.
- Existing unit, real-command, focused Rules/Decisions TUI tests, formatting, and strict Clippy pass.
- Visible Rules/Decisions behavior receives genuine human `just dev` validation if touched beyond invisible adapter wiring.
- Temporary lint expectations assigned to these operations are removed.
- No broad CLI/TUI decomposition, protocol change, release, or push occurs.

Creating this Task does not authorize starting it.
