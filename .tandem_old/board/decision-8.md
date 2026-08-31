---
id: decision-8
type: decision
title: "Define canonical protocol, project, app, and peer CLI/TUI architecture"
status: "accepted"
date: "2026-07-26"
deciders: ["Algorant"]
context: "Tandem's single Rust binary crate concentrates protocol meaning, concrete .tandem filesystem access, application orchestration, CLI presentation, and TUI behavior in oversized roots. The reviewed plan/refactor_spec.md records the project owner's resolved architecture and protocol 0.2 compatibility choices for Epic task-146, but the campaign requires a durable accepted architecture decision before implementation proceeds."
consequences: ["Repository protocol/ Markdown remains normative; tandem/src/protocol/ becomes its sole executable Rust implementation.", "project::TandemProject owns project discovery and concrete .tandem reads, preservation, locking, conflicts, atomic writes, archive moves, and event files without inferring protocol meaning.", "app owns shared typed use cases; CLI and TUI are peer interfaces over protocol and app behavior.", "The campaign remains one Cargo package and one production binary crate with no lib.rs, root workspace, generic storage traits, or new framework architecture.", "Protocol 0.2 compatibility changes land separately from behavior-preserving movement, with real-command regression tests and strict Clippy checkpoints.", "Rust CLI/TUI implementation work freezes on main while refactor/protocol-architecture is active; delegated Task work is reviewed before integration.", "Visible TUI stages require best-effort live-versus-dev parity validation, and unresolved visual judgment remains in validation."]
alternatives: ["Keep protocol, filesystem, application, CLI, and TUI responsibilities in main.rs and tui.rs; rejected because ownership and dependency direction remain unenforceable.", "Introduce multiple crates or a public Rust library first; rejected because the campaign preserves one binary crate until a real second Rust consumer exists.", "Create generic repository, storage, component, or dependency-injection abstractions during extraction; rejected because Tandem has no demonstrated second implementation.", "Redesign CLI parsing, output, TUI interaction, or protocol behavior during move-only stages; rejected because behavior changes must remain explicit and attributable."]
references: ["task-145", "task-146", "task-148", "task-149", "task-150", "task-151", "task-152", "task-153", "task-154", "task-155", "task-156", "task-157", "task-158", "task-159", "task-160", "task-161", "task-162", "task-163", "task-164", "decision-7"]
tags: ["architecture", "protocol", "rust", "cli", "tui", "refactor"]
createdAt: "2026-07-26T21:18:21Z"
updatedAt: "2026-07-26T21:18:21Z"
---

## Status

Accepted.

## Decision

Adopt the ownership and dependency model reviewed in `plan/refactor_spec.md` for Epic `task-146`:

```text
protocol <- project
protocol + project <- app
protocol + app <- cli
protocol + app <- tui
cli + tui startup <- main
```

Repository `protocol/` Markdown is normative. `tandem/src/protocol/` is its executable implementation. `project::TandemProject` owns concrete project-local filesystem behavior. `app` composes shared typed use cases. CLI and TUI remain peer interfaces, and `main.rs` remains process composition and exit wiring.

The target TUI module root is `tui/mod.rs`. Leaf files are created only when cohesive implementation moves into them; proposed filenames and line counts are review aids rather than protocol requirements.

## Compatibility and execution policy

The protocol 0.2 compatibility policy, branch strategy, strict lint policy, real-command tests, visibility constraints, and behavior-preservation checkpoints in `plan/refactor_spec.md` are accepted as campaign requirements. Protocol/product changes must remain separate from move-only extraction commits.

## Supersession

This decision does not supersede decision-7. Decision-7 remains authoritative for Epic, Task, Subtask, relationship, and role-specific ID semantics.
