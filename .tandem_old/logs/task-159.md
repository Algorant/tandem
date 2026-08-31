---
id: task-159
type: task
title: "Extract CLI parsing, dispatch, commands, and output ownership"
priority: "high"
parentId: "task-146"
blockers: ["task-158"]
references: ["task-145"]
relatedFiles: ["plan/refactor_spec.md", "tandem/src/main.rs", "tandem/src/cli/", "tandem/tests/cli_behavior.rs", "tandem/Cargo.toml"]
tags: ["cli", "rust", "architecture", "refactor"]
createdAt: "2026-07-22T20:42:30Z"
updatedAt: "2026-07-28T03:12:07Z"
accord:
  status: "accepted"
  assignee: "worker-task-159-278e210e"
  claimedAt: "2026-07-28T02:44:00Z"
  deliveredAt: "2026-07-28T03:11:10Z"
  deliverables: ["cli/args.rs manual parsers", "cli/commands.rs thin adapters", "cli/output.rs exact output ownership", "cli/mod.rs dispatch/startup request", "app project/query operations", "wiring-only main.rs", "direct-owner TUI imports"]
  validation:
    commands: ["206 unit tests passed", "6 real-command tests passed", "focused CLI behavior tests passed", "formatting and strict Clippy passed", "no production CLI persistence primitives", "no CLI-to-TUI or reverse dependencies", "no main compatibility re-exports", "no test loss"]
  summary: "Extracted CLI parsing, dispatch, command adapters, and exact output into a peer CLI interface with app-owned durable/query operations and a 55-line wiring-only main."
  filesChanged: ["tandem/src/app/decisions.rs", "tandem/src/app/mod.rs", "tandem/src/app/project.rs", "tandem/src/app/queries.rs", "tandem/src/cli/args.rs", "tandem/src/cli/commands.rs", "tandem/src/cli/mod.rs", "tandem/src/cli/output.rs", "tandem/src/main.rs", "tandem/src/project/mod.rs", "tandem/src/protocol/config.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/editor.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/review.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/theme.rs", "tandem/tests/cli_behavior.rs"]
  reviewer: "orchestrator"
  note: "Accepted after rework removed direct CLI persistence/protocol construction and main compatibility re-exports; independent full and exact process validation passed."
  updatedAt: "2026-07-28T03:11:17Z"
assignee: "worker-task-159-278e210e"
completedAt: "2026-07-28T03:12:07Z"
completion:
  summary: "Extracted CLI parsing, dispatch, thin command adapters, and exact output ownership; moved durable project/Decision operations and canonical queries into app; reduced main.rs to 55 lines of wiring/error handling. Integrated as cb3a69e and revalidated on target with 206 unit and 6 real-command tests, formatting, strict Clippy, exact process, persistence/import, reverse-dependency, and no-test-loss audits."
  filesChanged: ["tandem/src/app/decisions.rs", "tandem/src/app/mod.rs", "tandem/src/app/project.rs", "tandem/src/app/queries.rs", "tandem/src/cli/args.rs", "tandem/src/cli/commands.rs", "tandem/src/cli/mod.rs", "tandem/src/cli/output.rs", "tandem/src/main.rs", "tandem/src/project/mod.rs", "tandem/src/protocol/config.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/editor.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/review.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/theme.rs", "tandem/tests/cli_behavior.rs"]
  validation: "Target: cargo fmt --all --manifest-path tandem/Cargo.toml -- --check; cargo test --manifest-path tandem/Cargo.toml --no-fail-fast (206+6 passed); cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings; git diff --check; production CLI persistence audit; CLI-to-TUI/reverse-dependency audit; main re-export/lint audit; exact output/exit tests"
  reviewer: "orchestrator"
---

## Description

## Objective

Make CLI a thin peer interface over shared protocol/app behavior and reduce `main.rs` to composition, process entry, and exit handling.

## Scope

- Move manual argument records/parsers to `cli/args.rs`.
- Move help/version/dispatch to `cli/mod.rs`.
- Move thin command adapters into cohesive CLI command modules/files only where warranted.
- Move exact human tables/details, warnings, and JSON envelopes to `cli/output.rs`.
- Represent the `tui` command as a startup request composed by `main`; CLI must not import TUI implementation code.
- Preserve manual parsing, long-flag policy, command names, usage text, exit categories, stdout/stderr placement, field order, omission rules, escaping, and warning order.

## Acceptance criteria

- `main.rs` contains only module declarations/composition and process error/exit wiring, targeting roughly 100 lines without making size a CI rule.
- CLI performs no protocol inference or direct `.tandem` writes and TUI does not call printing command handlers.
- No `clap`, serialization framework, new production dependency, or output redesign is introduced.
- Parser/output unit tests, full tests, real-command exact-output tests, formatting, and strict Clippy pass.
- Temporary lint expectations assigned to CLI/main are removed.
- No TUI state/render split, protocol change, release, or push occurs.

Creating this Task does not authorize starting it.
