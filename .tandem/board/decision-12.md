---
id: decision-12
type: decision
title: "Adopt protocol 0.3.0 and a clap-derived minimal CLI"
status: "accepted"
date: "2026-08-30"
deciders: ["Algorant"]
context: "Sustained use showed that Tandem's 1,147-line handwritten parser and 39-command CLI duplicate record, lifecycle, help, output, and storage behavior. Help works on only two of 46 current surfaces, metadata lists cannot be replaced or cleared, and static help/tests already preserve drift. Task-246 completed an interactive audit of the CLI and adjacent protocol model."
consequences: ["Protocol 0.3.0 becomes the target convention for new workspaces; existing workspace handling remains project-owner work outside the cutover.", "Core CLI contracts to 23 invocable commands and uses clap 4.6.x derive with global JSON and generated help.", "Active Tasks require Accord acceptance criteria; Validation is reserved for exceptional human confirmation; Papercuts become tagged low-priority Tasks.", "Tasks, Decisions, Rules, Logs, and per-actor events use dedicated storage locations.", "Implementation occurs in one comprehensive isolated Task/branch with no dual parser, migration, compatibility reader, fallback, or partial merge.", "Pi integration is overhauled later in its owning workspace against the finished CLI; final intent-shaped tool inventory is decided there."]
alternatives: ["Patch the handwritten parser and help in place; rejected because grammar, documentation, errors, and repetition remain manually synchronized.", "Port the current 39-command grammar directly to clap; rejected because it preserves product duplication and one-off behavior.", "Split protocol, storage, CLI, and TUI into separately merged cutovers; rejected because heavily overlapping intermediate states would require compatibility paths or broken main.", "Use a builder-only or parallel derive/builder command model; rejected because one static derive tree is sufficient."]
references: ["task-246", "papercut-1", "papercut-9", "decision-3", "decision-8"]
tags: ["protocol", "cli", "clap", "architecture", "cutover"]
createdAt: "2026-08-30T17:47:25Z"
updatedAt: "2026-08-30T17:47:25Z"
---

## Status

Accepted.

## Context

Tandem's handwritten CLI parser spans 1,147 lines and repeats grammar, help, and error behavior across 39 commands. Sustained use exposed broader product duplication across Task, Log, Decision, Papercut, Accord, Review, storage, and completion models. The complete evidence and interactive decisions are recorded in `tandem/plan/clap-migration-research.md` and `plan/cli-protocol-cutover.md`.

## Decision

Adopt protocol 0.3.0 and the complete product/CLI contract in `plan/cli-protocol-cutover.md`.

Use clap 4.6.x with default features plus derive for one static command model. Remove the handwritten parser in one comprehensive cutover and do not ship dual parsers. Use the selected 23-command surface, unified lookup/update, separate structured list and full-text search, global JSON, generated help, mandatory active Task Accord, tagged-Task Papercuts, per-file Rules, typed storage directories, minimal Logs, and structured per-actor events.

Preserve Decision as first-class ADR content, fixed Epic → Task → Subtask roles, parent-derived Subtask IDs, Task state plus separate mandatory Accord status, and exceptional human Validation. Preserve the current TUI list/subview and State/Epic arrangement architecture with only required protocol adaptations.

Implement on one isolated Task branch and merge only the coherent candidate. The runtime supports protocol 0.3.0 only. Existing workspace handling remains outside the protocol and implementation plan.

## Compatibility policy

Correct new behavior replaces current command, flag, output, storage, and protocol compatibility. Do not ship upgrade/migrate commands, implicit conversion, compatibility readers, backup conventions, aliases or fallback paths for removed behavior.

Help/version remain generated text. Human results use stdout and warnings/errors use stderr. Global JSON uses stdout-only success/error envelopes. Exit statuses remain 0 success, 2 usage, and 1 operational failure.

## Consequences

PC1 and PC9 resolve in the core cutover. `app::Error` replaces app-layer coupling to `CliError`. Shell completions, manpages, web redesign, workspace handling, Pi adapter implementation, and the proposed contextual TUI lifecycle picker remain separate.

The isolated clap prototype added about 501 KB stripped and requires Rust 1.85 through the dependency, though Tandem does not declare an MSRV.

## Supersession and amendment

This Decision supersedes v0 guidance prohibiting clap and every current v0 CLI/protocol command, storage, and lifecycle decision that conflicts with the accepted cutover ledger. It does not supersede decision-8's protocol/project/app and peer-interface ownership direction. It amends decision-3's Rust stack by adding clap as the canonical CLI parser.

## Alternatives

The rejected alternatives are preserved in the research artifact and cutover ledger.
