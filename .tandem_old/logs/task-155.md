---
id: task-155
type: task
title: "Introduce project::TandemProject and concrete read boundaries"
priority: "high"
parentId: "task-146"
blockers: ["task-154"]
references: ["task-145"]
relatedFiles: ["plan/refactor_spec.md", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/project/", "protocol/plan/spec.md"]
tags: ["protocol", "rust", "architecture", "refactor"]
createdAt: "2026-07-22T20:41:41Z"
updatedAt: "2026-07-26T23:14:36Z"
accord:
  status: "accepted"
  assignee: "worker-task-155-88796a2f"
  claimedAt: "2026-07-26T22:58:55Z"
  deliveredAt: "2026-07-26T23:14:24Z"
  deliverables: ["tandem/src/project/mod.rs", "tandem/src/protocol/document.rs", "tandem/src/protocol/hierarchy.rs", "CLI/TUI read-boundary migrations"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml -- --check", "cargo test --manifest-path tandem/Cargo.toml --no-fail-fast: 183 unit + 4 executable tests passed", "cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings", "Dependency audit: no protocol imports from project/root filesystem boundaries"]
  summary: "Introduced TandemProject and concrete strict/tolerant read boundaries; separated protocol hierarchy inputs from project source/path adapters; moved lifecycle alias normalization into protocol."
  filesChanged: ["tandem/src/project/mod.rs", "tandem/src/protocol/document.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/theme.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/rules.rs"]
  reviewer: "orchestrator"
  note: "Accepted after rework and independent review. Strict/tolerant reads, protocol/project dependency direction, metadata normalization ownership, retained project paths, full tests, formatting, and strict Clippy all satisfy the task criteria."
  updatedAt: "2026-07-26T23:14:30Z"
assignee: "worker-task-155-88796a2f"
completedAt: "2026-07-26T23:14:36Z"
completion:
  summary: "Extracted TandemProject and concrete read boundaries while preserving strict/tolerant behavior and protocol/project dependency direction."
  filesChanged: ["tandem/src/project/mod.rs", "tandem/src/protocol/document.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/theme.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/rules.rs"]
  validation: "183 unit tests and 4 real-command tests passed; cargo fmt check and strict all-target Clippy passed; dependency audit confirmed protocol does not depend on project."
  reviewer: "orchestrator"
---

## Description

## Objective

Create the concrete filesystem-facing project boundary for discovering and reading one Tandem project while leaving protocol meaning in `protocol`.

## Scope

- Establish `project::TandemProject` with the discovered project root and resolved project-local `.tandem/` paths.
- Extract project-root/data-directory discovery, including documented compatibility paths.
- Extract raw Markdown/YAML access, source preservation, strict and tolerant Board/Log/config/event reads, and project snapshot inputs.
- Keep field meaning, supported values, hierarchy, workflow, accord, review, and diagnostic policy in protocol.
- Keep writes, locks, conflict checks, atomic replacement, event append, and archive moves for the next Task.
- Replace the legacy broad root `Workspace` ownership model without introducing repository/storage traits.

## Acceptance criteria

- `project` depends on protocol and never the reverse; it performs no role or lifecycle inference.
- Unknown fields and Markdown bodies remain available for byte-preserving/minimal later writes.
- Strict reads fail closed and tolerant TUI reads retain existing diagnostics/non-panicking behavior.
- Discovery/read tests move with implementation; real-command and TUI reload tests remain green.
- Formatting, full tests, real-command tests, strict Clippy, and dependency/visibility review pass.
- Temporary lint expectations assigned to project reads are removed.
- No write-boundary extraction, app/interface redesign, new trait, release, or push occurs.

Creating this Task does not authorize starting it.
