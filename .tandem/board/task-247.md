---
id: task-247
type: task
title: "Implement protocol 0.3.0 and the comprehensive clap CLI cutover"
state: todo
priority: "critical"
effort: "large"
references: ["task-246", "decision-12", "papercut-1", "papercut-9", "decision-3", "decision-8"]
relatedFiles: ["plan/cli-protocol-cutover.md", "tandem/plan/clap-migration-research.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/Cargo.toml", "tandem/src/protocol/", "tandem/src/project/", "tandem/src/app/", "tandem/src/cli/", "tandem/src/main.rs", "tandem/src/tui/", "tandem/src/web/", "tandem/tests/cli_behavior.rs", "tandem/README.md"]
tags: ["protocol", "cli", "clap", "cutover"]
createdAt: "2026-08-30T17:48:26Z"
updatedAt: "2026-08-30T17:48:26Z"
---

## Description

## Goal

Implement accepted `decision-12` and the complete contract in `plan/cli-protocol-cutover.md` as one comprehensive core cutover. Work in one isolated branch/worktree with ordered commits. Review and merge only the coherent candidate.

## Authoritative inputs

- `decision-12`
- `plan/cli-protocol-cutover.md`
- `tandem/plan/clap-migration-research.md`, especially sections 3–7
- `task-246` completed research Log

If implementation discovers a real contradiction, return for a new Decision. Do not silently alter the accepted contract.

## Required implementation order

1. Normative protocol 0.3.0 specification.
2. Rust protocol types and validation.
3. Project storage layout, discovery, events, and per-file Rules.
4. App operations, Accord lifecycle, archive behavior, and `app::Error`.
5. Clap dependency, derive model, parser/error bootstrap, and typed dispatch.
6. Unified human/JSON output, read scope, deterministic update replacement, and generated help.
7. Process, protocol, app, project, and generated-help tests.
8. Targeted TUI adaptation preserving current list/subview and State/Epic arrangement architecture.
9. Web read-model compatibility, docs, landing, release notes, and removal of superseded code.

## Core scope

- Protocol version `0.3.0` only.
- `.tandem/tasks/`, `decisions/`, `rules/`, `logs/`, and per-actor `events/` layout.
- First-class Task and Decision records; one Rule per Markdown file.
- Papercuts as low-priority Tasks tagged `papercut`.
- Fixed Epic → Task → Subtask hierarchy and current role-specific IDs.
- Mandatory Accord on active Tasks with required acceptance criteria.
- Task states `todo`, `in-progress`, `validation`; remove Review status.
- Validation only through explicit human escalation.
- Accord ready/claim/deliver/rework/block/resume/release/fail, with complete atomically accepting delivered work.
- Minimal archived resolution metadata without duplicated delivery evidence.
- Structured per-actor events for every durable mutation.
- Clap 4.6.x defaults + derive; no `rust-version` declaration.
- Exact 23-command target in the research artifact.
- Global `-j/--json`, generated `-h/--help`, `-V/--version`, unified output/error streams, and exit 0/1/2.
- Deterministic list replacement and generic `--clear` semantics.
- `app::Error` instead of app-layer `CliError` coupling.

## Acceptance

- Normative and executable protocol 0.3.0 semantics agree.
- Only the accepted target command tree parses; every removed command and flag fails with usage.
- All 27 root/family/leaf help surfaces work without a workspace.
- `tandem/src/cli/args.rs` and the handwritten parser are removed.
- CLI modules follow the accepted `model.rs`, `parse.rs`, `commands.rs`, `output.rs`, `landing.rs`, wiring-only `mod.rs` ownership.
- Global JSON handles every command and grammar/operational failure using the canonical envelope.
- Human results use stdout; warnings/errors use stderr.
- PC1 and PC9 are resolved exactly as specified.
- New storage directories and per-file Rules are the only runtime path.
- No upgrade/migrate command, converter, compatibility reader, backup convention, alias shim, dual parser, or fallback ships.
- TUI keeps Board/Logs/Rules/Decisions, current state subviews/full-width rows, and State/Epic arrangement; Papercuts becomes the fourth Board section; Add/direct Move are removed; Validation is adapted.
- Web compiles and reads the new app/storage model; visual redesign remains deferred.
- `cargo test`, format/lint, release build, and real-command smoke suite pass.
- Actual clean/cached build and stripped release-size delta are recorded.
- No adapter implementation or existing-workspace transition work enters this Task.

## Rollback boundary

The isolated branch is the rollback boundary. If the candidate fails review, discard or rework it. Never merge a partial cutover or compatibility path.
