---
id: decision-8
type: decision
title: "Adopt protocol 0.3.0 and a clap-derived minimal CLI"
status: "accepted"
deciders: ["Algorant"]
tags: ["protocol", "cli", "clap", "architecture", "cutover"]
createdAt: "2026-08-31T20:52:47Z"
decidedAt: "2026-07-26T00:00:00Z"
updatedAt: "2026-08-31T20:52:47Z"
---

## Status

Accepted.

## Context

Tandem's handwritten CLI parser spanned 1,147 lines and repeated grammar, help, and error behavior across 39 commands; help worked on only two of 46 surfaces, list fields could not be replaced or cleared, and sustained use exposed broader duplication across Task, Log, Decision, Papercut, Accord, Review, storage, and completion models. This decision records the resulting protocol 0.3.0 + CLI cutover and its consequences. It supersedes every conflicting pre-0.3.0 CLI/protocol, storage, and lifecycle decision; it amends the stack decision by adding clap as the canonical CLI parser; it does not supersede the protocol/project/app/peer-interface architecture decision.

## Decision

Adopt protocol 0.3.0 and the complete product/CLI contract in `plan/cli-protocol-cutover.md`. Use clap 4.6.x with derive for one static command model; remove the handwritten parser in one comprehensive cutover with no dual parser. Use the selected 23-command surface with typed `add`, `accord`, and `rules` families; unified lookup/update; separate structured `list` and full-text `search`; global JSON; generated help; mandatory active-Task Accord; Papercuts as tagged low-priority Tasks; per-file Rules; typed storage directories (`tasks/`, `decisions/`, `rules/`, `logs/`, per-actor `events/`); minimal archived Logs with `resolution` metadata; and structured per-actor events.

Preserve Decision as first-class ADR content; fixed Epic → Task → Subtask roles; parent-derived Subtask IDs; Task state plus a separate mandatory Accord status; and exceptional human Validation. Preserve the TUI list/subview and State/Epic arrangement architecture with only the required protocol adaptations.

Implement on one isolated Task branch; merge only the coherent candidate. The runtime supports protocol 0.3.0 only; older workspaces fail clearly. Existing workspace handling remains project-owner work outside the protocol.

## Compatibility policy

Correct new behavior replaces current compatibility. No upgrade/migrate command, implicit conversion, compatibility reader, backup convention, alias shim, dual parser, or old protocol path ships. Help/version remain generated text; human results use stdout and warnings/errors use stderr; global JSON uses stdout-only success/error envelopes; exit statuses remain 0 success, 2 usage, and 1 operational failure.

## Consequences

- PC1 (per-command help) and PC9 (list replacement/clear) resolve in the cutover.
- `app::Error` replaces app-layer coupling to process-oriented CLI errors.
- TUI/Web/docs/tests require broad aligned updates.
- Shell completions, manpages, web redesign, workspace migration, and the Pi adapter overhaul remain separate.

## Alternatives

Rejected alternatives are preserved in the cutover research and ledger: direct port of the old grammar, dual parsers, protocol fallbacks, a separate Papercut protocol, Review status, routine Validation, accepted-but-active Tasks, optional Accord, duplicate assignees, shared events JSONL, Rules in shared config, kanban TUI redesign, and separately merged cutover stages.
