---
id: task-157
type: task
title: "Unify Task, accord, Validation, and completion application operations"
priority: "high"
parentId: "task-146"
blockers: ["task-156"]
references: ["decision-7"]
relatedFiles: ["plan/refactor_spec.md", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/app/tasks.rs", "tandem/src/app/accord.rs"]
tags: ["protocol", "cli", "tui", "rust"]
createdAt: "2026-07-22T20:42:03Z"
updatedAt: "2026-07-28T02:17:59Z"
accord:
  status: "accepted"
  assignee: "worker-task-157-8b929283"
  claimedAt: "2026-07-28T01:57:51Z"
  deliveredAt: "2026-07-28T02:17:14Z"
  deliverables: ["Shared app Task lifecycle operations", "Shared app accord transition operations", "Shared Validation and completion/archive operations", "Final architecture and interface-parity audit"]
  validation:
    commands: ["200 unit tests passed", "4 real-command tests passed", "cargo fmt passed", "strict Clippy passed", "ownership and reverse-dependency audits passed", "human just dev Validation behavior approved"]
  summary: "Unified Task, accord, Validation, and completion application operations with shared CLI/TUI app boundaries; all five checkpoints completed and independently validated."
  evidence: ["Commits through 363d559", "All task-157 subtasks completed"]
  filesChanged: ["tandem/src/app/accord.rs", "tandem/src/app/mod.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/main.rs", "tandem/src/project/frontmatter.rs", "tandem/src/project/mod.rs", "tandem/src/protocol/accord.rs", "tandem/src/protocol/workflow.rs", "tandem/src/tui/mod.rs"]
  reviewer: "orchestrator"
  note: "Accepted after independent final architecture review, full automated validation, completed child checkpoints, and human visible Validation approval."
  updatedAt: "2026-07-28T02:17:17Z"
assignee: "worker-task-157-8b929283"
completedAt: "2026-07-28T02:17:59Z"
completion:
  summary: "Unified Task, accord, Validation, and completion application operations across shared app boundaries. All five checkpoints completed; Worker changes integrated as cba6cdf into refactor/protocol-architecture and revalidated on target with 200 unit tests, 4 real-command tests, formatting, strict Clippy, ownership/interface audits, and human visible Validation approval."
  filesChanged: ["tandem/src/app/accord.rs", "tandem/src/app/mod.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/main.rs", "tandem/src/project/frontmatter.rs", "tandem/src/project/mod.rs", "tandem/src/protocol/accord.rs", "tandem/src/protocol/workflow.rs", "tandem/src/tui/mod.rs"]
  validation: "Target branch: cargo fmt --all --manifest-path tandem/Cargo.toml -- --check; cargo test --manifest-path tandem/Cargo.toml --no-fail-fast (200+4 passed); cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings; git diff --check; shared CLI/TUI app reachability and lifecycle ownership audits; human just dev approval"
  reviewer: "orchestrator+human"
---

## Description

## Objective

Move command-independent Task lifecycle use cases into `app` and make CLI and TUI consume the same typed operations.

## Scope

- Establish shared app inputs/outcomes for Task add, move, metadata update, accord transitions/state synchronization, Validation accept/rework, and completion/archive.
- Compose canonical protocol validation with concrete TandemProject reads/writes.
- Return typed results, warnings, and diagnostics without printing, rendering, parsing process arguments, or managing transient UI state.
- Switch CLI and TUI call sites for each mutation family before proceeding to the next; remove duplicated orchestration.
- Preserve exact files, events, warnings, exit behavior, reload behavior, and hierarchy safety.
- Use concrete types/functions; do not introduce service/repository traits or dependency injection.

## Acceptance criteria

- CLI and TUI invoke the same app operation for every covered durable mutation.
- No printing/Ratatui/process parsing enters `app`; no protocol meaning enters project or interfaces.
- Representative operations produce byte/semantic-equivalent documents, events, warnings, and outcomes through both interfaces.
- Unit, real-command, concurrency, focused Validation/TUI tests, formatting, and strict Clippy pass.
- Visible Validation behavior receives genuine human `just dev` approval.
- Temporary lint expectations assigned to these use cases are removed.
- No Rules/Decision migration, broad TUI/CLI split, release, or push occurs.

Creating this Task does not authorize starting it.
