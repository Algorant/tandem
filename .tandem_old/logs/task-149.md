---
id: task-149
type: task
title: "Establish executable behavior and strict lint guardrails"
priority: "high"
parentId: "task-146"
blockers: ["task-148"]
references: ["task-145"]
relatedFiles: ["plan/refactor_spec.md", "tandem/Cargo.toml", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/tests/"]
tags: ["cli", "rust", "testing", "refactor"]
createdAt: "2026-07-22T20:40:21Z"
updatedAt: "2026-07-26T21:31:20Z"
accord:
  status: "accepted"
  assignee: "worker-task-149-1b88d11a"
  claimedAt: "2026-07-26T21:21:22Z"
  deliveredAt: "2026-07-26T21:31:05Z"
  deliverables: ["tandem/tests/cli_behavior.rs with 3 real-binary process tests", "21 safe lint fixes", "8 narrow checkpoint-named too_many_arguments expectations"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml -- --check", "cargo test --manifest-path tandem/Cargo.toml: 167 unit + 3 executable tests passed", "cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings", "git show --check 4a0d9c4", "clean Worker status"]
  summary: "Added executable real-command behavior coverage and established a green strict-Clippy baseline while preserving behavior."
  evidence: ["167 unit and 3 executable tests passed", "strict Clippy passed with -D warnings", "format check passed", "8 narrow expects audited; no broad allowance", "commit 4a0d9c4 fast-forwarded into integration branch"]
  filesChanged: ["tandem/tests/cli_behavior.rs", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/review.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/theme.rs"]
  reviewer: "parent-orchestrator"
  note: "Reviewed the complete diff and independently reran all required validation on the exact Worker commit. The process tests cover help/version/usage, missing project/document, human/JSON reads, add/move/update/complete/accord, hierarchy success/failure, raw preservation, events, and logs. Lint edits are behavior-equivalent; all remaining expectations are item-local and name Stage 6 removal checkpoints."
  updatedAt: "2026-07-26T21:31:12Z"
assignee: "worker-task-149-1b88d11a"
completedAt: "2026-07-26T21:31:20Z"
completion:
  summary: "Added deterministic executable Tandem CLI regression tests and established a strict-Clippy-clean campaign baseline. Covered process contracts, representative reads/mutations, hierarchy failures, preservation, events, completion, and Logs; safely fixed 21 diagnostics and isolated 8 Board/Review argument-count diagnostics with precise Stage 6 removal checkpoints."
  filesChanged: ["tandem/tests/cli_behavior.rs", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/review.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/theme.rs"]
  validation: "Parent independently ran cargo fmt --check, cargo test (167 unit + 3 executable tests), and cargo clippy --all-targets -- -D warnings on commit 4a0d9c4; all passed. Full diff and all 8 item-level expectations reviewed."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Create the behavior and lint baseline that every later extraction Task must preserve on `refactor/protocol-architecture`.

## Scope

- Add tests under `tandem/tests/` that run the compiled `tandem` executable in temporary projects and inspect status, stdout, stderr, and `.tandem/` effects.
- Cover help/version, usage failures, missing project/document, representative human/JSON reads, add/move/update/complete/accord flows, hierarchy success/failure, unknown-field/body preservation, events, and completed logs.
- Inventory all existing strict-Clippy diagnostics.
- Fix safe local diagnostics or isolate unavoidable legacy debt with the narrowest item-level `#[expect]` and a named removal checkpoint.
- Prohibit crate-wide warning allowances and ensure new warnings remain visible.

## Acceptance criteria

- Real-command tests are deterministic, create their own temporary project data, and add no fixture directory or production dependency.
- Existing CLI output, exit categories, persisted data, and event behavior are unchanged.
- `cargo fmt --check`, the full test suite, the new executable tests, and `cargo clippy --all-targets -- -D warnings` pass.
- Every temporary expectation is precise, justified, and mapped to a later module checkpoint.
- No module extraction, protocol 0.2 behavior change, TUI redesign, release, or push occurs.

Creating this Task does not authorize starting it.
