# Tandem Parent Todo

Status: active implementation
Last updated: 2026-06-28

This todo tracks monorepo-level work that cuts across the protocol, TUI, and extensions areas.

## Accomplished

- [x] Chose working project/protocol name: **Tandem**.
- [x] Chose `accord` as the replacement for Brainfile's `contract` object.
- [x] Established `tandem` as the user-facing CLI and reserved `td` for future/internal tool prefixes.
- [x] Renamed local repository to `/home/ivan/dev/projects/tandem`.
- [x] Created private GitHub repo: `Algorant/tandem`.
- [x] Set initial monorepo direction with `protocol/` and `tandem/` areas.
- [x] Created initial protocol and TUI specs.
- [x] Moved specs into `plan/spec.md` files.
- [x] Added root project planning spec.
- [x] Added documentation contract requiring README/spec/todo docs for every discrete project aspect.
- [x] Added `protocol/README.md` and `tandem/README.md`.
- [x] Locked v0 reconciliation decisions for protocol fields, lifecycle, logs/events, CLI commands, and TUI MVP scope.
- [x] Resolved remaining protocol detail decisions: `protocolVersion: 0.1.0`, minimal audit-only events, strict-core-reference validation severity, and no v0 decision lifecycle field.
- [x] Resolved CLI/TUI detail decisions: `tandem tui` only, `tandem decision list|show|add`, no short aliases, compact-table/detail-block human output, `--json` envelope objects for all read commands, TOML theme loading built-in/user/workspace, mouse on by default without drag/drop, fixed v0 keymaps, styled-basic Markdown rendering, and simple filtered Review queue.
- [x] Resolved parent strategy decisions: keep implementation under `tandem/` for v0, no schemas/fixtures in v0, migrate/dogfood Tandem documents after the tooling can manage them safely, and reserve `td` while keeping `tandem` user-facing.
- [x] Accepted protocol v0 draft for implementation.
- [x] Implemented and hardened the current known v0 `tandem` CLI surface inside `tandem/`.
- [x] Started the first Ratatui/crossterm `tandem tui` implementation with a read-only Board shell under `tandem/src/tui.rs`.
- [x] Expanded `tandem tui` to Board, Review, Logs, Rules, and Decisions views with initial Board, Rules, Decisions, and Logs interactions.
- [x] Reworked the Board into count-labeled state subviews with a full-width selected-state list and richer rows while preserving quick-add and `H`/`L` task moves.
- [x] Added first-class TUI theme discovery from `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, plus workspace `theme = "name"` selection and `default-dark`/`verdigris` preset examples.
- [x] Added `extensions/` as the third major area for agent/editor integrations.
- [x] Added the initial `extensions/pi-tandem` Pi adapter MVP over installed `tandem`.

## Current tasks

- [x] Reconcile protocol docs against live Brainfile protocol plus `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md` enough for v0 implementation.
- [x] Build a Brainfile CLI/TUI feature parity and improvement matrix before implementation decisions.
- [ ] Keep Markdown planning docs canonical until the TUI can safely manage Tandem documents; migrate/dogfood Tandem documents after that.
- [x] Define minimal Rust implementation layout inside `tandem/` as implementation begins.
- [x] Decide exact Rust package/module layout for `tandem` inside `tandem/` from the first implementation slice.
- [ ] Revisit standalone `tandem` only after v0 if packaging/user needs justify it.
- [ ] Do not create schemas or fixtures in v0; revisit after TUI MVP/protocol stabilization.
- [ ] Track any remaining open naming/vocabulary decisions after detailed spec updates.
- [ ] Test `pi-tandem` as a project-local Pi extension before any global Pi config promotion.

## Task tag convention

Use task tags as lightweight delegation/filter hints, not as protocol schema.

- Put one primary area tag first: `protocol`, `tui`, `pi-tandem`, `docs`, `config`, `rules`, or `ui`.
- Add only a few capability/workflow tags when they help filtering, such as `accord`, `review`, `logs`, `editor`, `relationships`, `delegation`, `taxonomy`, `smoke`, or `validation`.
- Concrete TUI facet tags like `theme`, `keyboard`, `mouse`, and `markdown` are fine when they route work better than a broad `tui` tag alone.
- Keep tags lowercase/kebab-case and avoid meta tags like `mvp` or `polish` unless they materially help delegation.

## Next recommended steps

1. Add safe Review action buttons/mutations, likely accord accept/rework and completion/archive prompts.
2. Continue Board mutations after quick-add and move/change-state, likely edit, complete, or accord actions.
3. Continue remaining TUI polish without adding schemas or fixtures in v0.
4. Smoke `pi-tandem` in a real Pi session with project-local loading, then consider global Pi config promotion only as a separate reviewed step.
5. Migrate/dogfood Tandem documents after the TUI can manage them safely.

## Open questions

All currently listed parent strategy questions are resolved. Remaining work is tracked as implementation/planning tasks above.
