# Tandem Parent Todo

Status: active planning  
Last updated: 2026-06-26

This todo tracks monorepo-level work that cuts across the protocol and TUI areas.

## Accomplished

- [x] Chose working project/protocol name: **Tandem**.
- [x] Chose `accord` as the replacement for Brainfile's `contract` object.
- [x] Established `tdm` as the user-facing CLI and reserved `td` for future/internal tool prefixes.
- [x] Renamed local repository to `/home/ivan/dev/projects/tandem`.
- [x] Created private GitHub repo: `Algorant/tandem`.
- [x] Set monorepo direction with `protocol/` and `tandem-tui/` areas.
- [x] Created initial protocol and TUI specs.
- [x] Moved specs into `plan/spec.md` files.
- [x] Added root project planning spec.
- [x] Added documentation contract requiring README/spec/todo docs for every discrete project aspect.
- [x] Added `protocol/README.md` and `tandem-tui/README.md`.
- [x] Locked v0 reconciliation decisions for protocol fields, lifecycle, logs/events, CLI commands, and TUI MVP scope.
- [x] Resolved remaining protocol detail decisions: `protocolVersion: 0.1.0`, minimal audit-only events, strict-core-reference validation severity, and no v0 decision lifecycle field.
- [x] Resolved CLI/TUI detail decisions: `tdm tui` only, `tdm decision list|show|add`, no short aliases, compact-table/detail-block human output, `--json` envelope objects for all read commands, TOML theme loading built-in/user/workspace, mouse on by default without drag/drop, fixed v0 keymaps, styled-basic Markdown rendering, and simple filtered Review queue.
- [x] Resolved parent strategy decisions: keep implementation under `tandem-tui/` for v0, no schemas/fixtures in v0, migrate/dogfood Tandem documents after CLI MVP, and reserve `td` while keeping `tdm` user-facing.
- [x] Accepted protocol v0 draft for implementation.

## Current tasks

- [ ] Keep parent project plan aligned with `protocol/README.md`, `protocol/plan/spec.md`, `tandem-tui/README.md`, and `tandem-tui/plan/spec.md`.
- [x] Reconcile protocol docs against live Brainfile protocol plus `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md` enough for v0 implementation.
- [x] Build a Brainfile CLI/TUI feature parity and improvement matrix before implementation decisions.
- [ ] Enforce no-drift documentation updates whenever naming, scope, architecture, lifecycle, or workflow decisions change.
- [ ] Keep Markdown planning docs canonical until CLI MVP; migrate/dogfood Tandem documents after CLI MVP.
- [ ] Define minimal Rust implementation layout inside `tandem-tui/` as implementation begins.
- [ ] Decide exact Rust package/module layout for `tdm` inside `tandem-tui/` from the first implementation slice.
- [ ] Revisit standalone `tdm-tui` only after v0 if packaging/user needs justify it.
- [ ] Do not create schemas or fixtures in v0; revisit after CLI MVP/protocol stabilization.
- [ ] Track any remaining open naming/vocabulary decisions after detailed spec updates.

## Next recommended steps

1. Implement the first `tdm` CLI slice in `tandem-tui/`: `init`, `list`, and `show`.
2. Keep schemas and fixtures out of v0; use inline examples in docs.
3. Keep area READMEs, specs, and todos synchronized as implementation begins.
4. Migrate/dogfood Tandem documents after the CLI MVP can manage them safely.

## Open questions

All currently listed parent strategy questions are resolved. Remaining work is tracked as implementation/planning tasks above.
