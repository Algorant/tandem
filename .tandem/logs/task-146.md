---
id: task-146
type: task
kind: "epic"
title: "Refactor Tandem around a canonical protocol and peer CLI/TUI interfaces"
priority: "high"
references: ["task-145", "decision-7", "decision-8"]
relatedFiles: ["plan/refactor_spec.md", "tandem/plan/modularization-research.md", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/", "protocol/plan/spec.md", "tandem/plan/spec.md", "AGENTS.md"]
tags: ["protocol", "rust", "architecture", "refactor", "cli", "tui"]
createdAt: "2026-07-22T19:04:53Z"
updatedAt: "2026-07-29T00:51:25Z"
completedAt: "2026-07-29T00:51:25Z"
completion:
  summary: "Completed the canonical protocol/project/app/CLI/TUI architecture refactor campaign; all 18 direct Tasks completed and exact campaign history merged into local main at 0fa4955."
  filesChanged: ["protocol/", "tandem/", "extensions/pi-tandem/", "AGENTS.md", "README.md", "plan/refactor_spec.md"]
  validation: "Final comprehensive automated, structural, documentation, extension, packaging, PTY, and direct TUI validation passed; strict Clippy reports zero diagnostics and local main contains all 34 campaign commits."
  reviewer: "orchestrator"
---

## Description

## Outcome

Refactor Tandem’s existing single Rust binary crate on a dedicated integration branch so protocol semantics, concrete Tandem-project file access, shared application operations, CLI presentation, and TUI behavior have explicit, enforceable ownership.

## Target architecture

- Keep repository `protocol/` Markdown as the normative protocol specification.
- Establish `tandem/src/protocol/` as its sole executable Rust implementation.
- Establish `project::TandemProject` as the concrete owner of project-root discovery and project-local `.tandem/` file access, raw preservation, locking, atomic writes, and events.
- Retain `app` for shared use cases and typed outcomes.
- Treat `cli` and `tui` as peer interfaces over shared protocol/application behavior.
- Move the TUI root to `tui/mod.rs` and permit cohesive helper/feature modules beneath it.

## Campaign shape

Decompose this Epic into independently managed direct Tasks for architecture governance, behavior/lint guardrails, explicit protocol compatibility changes, low-risk TUI module setup, protocol extraction, TandemProject reads/writes, shared app operations, CLI extraction, TUI decomposition, documentation/agent-guidance alignment, and final validation/merge. Direct Epic children must use global Task IDs and explicit blockers; do not author inline checklist subtasks.

## Constraints

- One Cargo package and one production binary crate; no root workspace, extra package, `lib.rs`, or public Rust API during this campaign.
- Develop on `refactor/protocol-architecture`; freeze Rust CLI/TUI implementation work on `main` while it is active, except explicitly coordinated critical fixes.
- Keep movement separate from behavior redesign.
- Preserve protocol data, CLI output, events, TUI behavior, keybindings, themes, and release packaging unless a dedicated accepted protocol/product change explicitly says otherwise.
- Use precise temporary lint expectations only for known legacy diagnostics; strict Clippy remains green and all temporary suppressions are removed before merge.
- Add tests that run the compiled `tandem` command in temporary projects.
- Require human `just dev` validation for visible TUI stages.

## Completion criteria

- Protocol, project, app, CLI, and TUI boundaries match the accepted architecture decision.
- `main.rs` and `tui/mod.rs` are wiring roots rather than behavior warehouses.
- One canonical implementation owns hierarchy, IDs, workflow, accord, review, events, and structural diagnostics.
- CLI and TUI use shared app mutations; project owns concrete `.tandem` filesystem operations without protocol inference.
- Strict Clippy reports zero diagnostics, all temporary refactor suppressions are removed, automated behavior/regression tests pass, visible TUI behavior is human-approved, documentation and agent guidance match final paths, and the integration branch is merged cleanly to `main`.

Creating this Epic and later child Tasks does not authorize implementation; each Task starts only when explicitly requested.
