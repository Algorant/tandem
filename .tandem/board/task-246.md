---
id: task-246
type: task
title: "Research Clap migration and specify a comprehensive CLI cutover"
state: "in-progress"
priority: "high"
effort: "medium"
references: ["papercut-1", "papercut-9", "decision-3", "decision-8"]
relatedFiles: ["tandem/Cargo.toml", "tandem/src/cli/mod.rs", "tandem/src/cli/args.rs", "tandem/src/cli/commands.rs", "tandem/src/cli/landing.rs", "tandem/src/cli/output.rs", "tandem/tests/cli_behavior.rs", "tandem/plan/spec.md", "tandem/plan/todo.md", "tandem/plan/modularization-research.md", "extensions/pi-tandem/", "AGENTS.md"]
tags: ["config", "cli", "research", "clap"]
createdAt: "2026-08-30T12:30:45Z"
updatedAt: "2026-08-30T17:42:36Z"
accord:
  status: "delivered"
  assignee: "pi"
  claimedAt: "2026-08-30T13:50:09Z"
  deliveredAt: "2026-08-30T17:42:36Z"
  deliverables: ["plan/cli-protocol-cutover.md", "tandem/plan/clap-migration-research.md", "tandem/plan/spec.md link"]
  validation:
    commands: ["git diff --check", "All 10 required artifact sections present", "No pending phase placeholders remain", "Planning artifact links resolve", "Disposable clap 4.6.6 derive prototype built and exercised", "Target grammar/help/JSON/value edge cases prototyped"]
  summary: "Completed the interactive CLI/protocol audit and comprehensive clap cutover specification. Produced the authoritative protocol 0.3.0 decision ledger, full current grammar inventory, selected 23-command target, clap 4.6.6 prototype evidence, exact cutover/validation plan, proposed first-class Decision, and core/Pi handoff Tasks."
  evidence: ["Commit 3d82267 contains the completed research candidate", "Sideshow final candidate: http://localhost:8228/session/TgM6iL-l_-A/p/iFI5r_sqMJQ", "Clap prototype confirmed nested help, global JSON placement, leading-hyphen prose, duplicate scalar errors, repeatable lists, empty rejection, and contextual errors", "Measured isolated stripped clap delta 500752 bytes and cold release build about 5.1 seconds"]
  filesChanged: ["plan/cli-protocol-cutover.md", "tandem/plan/clap-migration-research.md", "tandem/plan/spec.md"]
  updatedAt: "2026-08-30T17:42:36Z"
assignee: "pi"
---

## Description


## Goal

Research whether Tandem should replace its handwritten CLI parser with `clap`, and produce the exact design and implementation plan for one comprehensive cutover. This is a research and specification task only. Do not add `clap`, change command behavior, or begin the migration.

The next step after owner review will be a dedicated comprehensive cutover Task. Before that implementation starts, record the selected direction as a first-class Tandem Decision and create explicit handoff Tasks in every affected Tandem workspace, including the Pi configuration/extensions workspace where Tandem command shapes and tool adapters are maintained.

## Context

Tandem currently has no `clap` dependency. Its CLI is handwritten across approximately 1,147 lines of argument parsing and 49 distinct long flags, with roughly 17 command families and multiple nested subcommand families.

Papercut-1 exposed a broader inconsistency: `tandem <command> --help` is promised by the landing page but fails for almost every command and nested command family. Only selected paths such as `web --help` work. Papercut-9 exposes another pending command-model question: list fields can be appended but not replaced, cleared, or selectively reduced.

Current project guidance explicitly says not to introduce `clap` in v0. The completed modularization research said to preserve manual parsing during code movement and evaluate replacements later as a separate product decision. This task is that evaluation. It must determine whether the original restriction still serves Tandem and define how to supersede it explicitly if adoption is recommended.

## Research questions

### Current grammar and compatibility inventory

Build a complete authoritative inventory of:

- every command and nested subcommand;
- positional arguments, required and optional flags, repeatable flags, defaults, and accepted values;
- command and subcommand help/version behavior;
- unknown-command, unknown-flag, missing-value, conflicting-option, and missing-workspace behavior;
- exit codes and whether parsing errors are returned or printed;
- `--json` placement and output contracts;
- values intentionally allowed to begin with `-`, especially Markdown bodies, notes, reasons, titles, and summaries;
- canonical long-flag-only and no-alias policy;
- workspace discovery boundaries, including help/version paths that must work without a Tandem workspace;
- parser behavior depended on by real-command tests, docs, scripts, release automation, and Pi adapters.

Identify accidental behavior that should not be preserved and mark every intentional compatibility requirement explicitly.

### Clap design

Evaluate a concrete current `clap` version and feature set. Decide and justify:

- derive API, builder API, or a narrow combination;
- the typed root command/subcommand/arguments structure;
- how nested families (`papercut`, `log`, `accord`, `review`, `rules`, `decision`) map to enums;
- how parser structs convert into existing `app` option types without moving protocol or filesystem behavior into CLI parsing;
- help layout, command descriptions, examples, version output, and error presentation;
- long flags only, with no implicit short aliases;
- `allow_hyphen_values`, trailing values, repeated options, optional values, and empty-string behavior where required;
- value parsers/enums versus app/protocol validation ownership;
- how `CliError`, process exit codes, and `StartupRequest::{Exit,Tui,Web}` interact with `clap::try_parse_from` or an equivalent path;
- whether shell completions or manpages belong in the cutover or remain deferred;
- dependency, compile-time, binary-size, MSRV, license, and release implications.

Prototype difficult parser shapes in disposable or test-only form if needed, but do not commit a production dependency or migration code.

### Comprehensive cutover design

Specify one authoritative cutover that removes the handwritten parser rather than retaining dual parsing paths or compatibility fallbacks. Define:

- exact files and modules to add, replace, shrink, or remove;
- command-model ownership and dependency direction;
- migration order inside one reviewable implementation branch;
- tests that compare the old documented contract to the new parser before the old path is removed;
- generated help coverage for every command and subcommand;
- documentation and landing-page updates;
- error/output compatibility decisions;
- PC1 resolution criteria;
- whether PC9 list replacement/removal/clearing syntax is included in the same cutover, decided immediately beforehand, or remains a separate product Task;
- release-note and compatibility implications;
- rollback boundary if the cutover fails before integration, without shipping two parsers.

The implementation plan must be detailed enough to create one comprehensive cutover Task with objective acceptance criteria and realistic effort.

### Decision and cross-workspace coordination

Identify every consumer affected by parser/help/error changes, including at minimum:

- core Tandem CLI/TUI startup;
- `extensions/pi-tandem/` in this repository;
- the canonical Pi configuration/extensions workspace under `~/.dotfiles/pi`;
- docs, scripts, release automation, examples, or tests that construct Tandem argument arrays or inspect help/error output;
- any other discovered Tandem workspace or adapter with a current dependency.

For each affected workspace, record:

- exact compatibility surface;
- whether no change is needed and why;
- required implementation and validation work;
- a ready-to-create Task title, body, references, related files, blockers, and acceptance criteria.

Do not mutate external workspaces during research. The owner will review the handoff map first; approved Tasks are then created in their owning Tandem workspaces before core cutover implementation.

Draft the proposed first-class Tandem Decision with Context, Decision, Consequences, Alternatives, compatibility policy, and supersession of the current no-`clap` v0 guidance. Do not mark it accepted without owner review.

## Required artifact

Create `tandem/plan/clap-migration-research.md` containing:

1. Executive recommendation.
2. Current CLI grammar and compatibility matrix.
3. Clap design with concrete type/module sketches.
4. Difficult-case prototype results.
5. Exact cutover specification and task-ready acceptance criteria.
6. Test and validation plan.
7. PC1 and PC9 dispositions.
8. Decision draft.
9. Cross-workspace impact and ready-to-create handoff Tasks.
10. Risks, rejected alternatives, and unresolved owner questions.

Update planning/spec documents only where needed to link this research artifact. Do not encode the recommendation as accepted product behavior before review.

## Acceptance

- The artifact inventories every current command, subcommand, positional, and flag.
- Help, error, exit-code, workspace-discovery, JSON, repeated-list, and flag-looking-value behavior are explicitly covered.
- The recommended `clap` architecture is concrete enough to implement without rediscovering parser design.
- The cutover removes the manual parser and does not ship dual paths.
- PC1 has clear resolution criteria and PC9 has an explicit sequencing decision.
- A complete proposed Tandem Decision is ready for owner review.
- Every affected workspace has either a justified no-change finding or a task-ready handoff specification.
- The follow-up core cutover Task can be created directly from the artifact with bounded scope and objective validation.
- No production dependency, CLI behavior, external workspace, or accepted Decision changes during this research Task.

