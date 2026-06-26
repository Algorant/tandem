# Tandem Parent Todo

Status: active planning  
Last updated: 2026-06-26

This todo tracks monorepo-level work that cuts across the protocol and TUI areas.

## Accomplished

- [x] Chose working project/protocol name: **Tandem**.
- [x] Chose `accord` as the replacement for Brainfile's `contract` object.
- [x] Established shorthand direction: `tdm` for the CLI, `td`/`tdm` for future integration prefixes.
- [x] Renamed local repository to `/home/ivan/dev/projects/tandem`.
- [x] Created private GitHub repo: `Algorant/tandem`.
- [x] Set monorepo direction with `protocol/` and `tandem-tui/` areas.
- [x] Created initial protocol and TUI specs.
- [x] Moved specs into `plan/spec.md` files.
- [x] Added root project planning spec.
- [x] Added documentation contract requiring README/spec/todo docs for every discrete project aspect.
- [x] Added `protocol/README.md` and `tandem-tui/README.md`.
- [x] Locked v0 reconciliation decisions for protocol fields, lifecycle, logs/events, CLI commands, and TUI MVP scope.

## Current tasks

- [ ] Keep parent project plan aligned with `protocol/README.md`, `protocol/plan/spec.md`, `tandem-tui/README.md`, and `tandem-tui/plan/spec.md`.
- [ ] Reconcile protocol docs against live Brainfile protocol plus `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md`.
- [ ] Build a Brainfile CLI/TUI feature parity and improvement matrix before implementation decisions.
- [ ] Enforce no-drift documentation updates whenever naming, scope, architecture, lifecycle, or workflow decisions change.
- [ ] Decide whether root `plan/todo.md` remains the canonical high-level tracker or whether Tandem should dogfood its own protocol once bootstrapped.
- [ ] Define minimal Rust implementation layout inside `tandem-tui/` when implementation begins.
- [ ] Decide exact Rust package/module layout for `tdm` inside `tandem-tui/`.
- [ ] Decide later whether standalone `tdm-tui` is needed beyond `tdm tui`.
- [ ] Create examples/fixtures only after the protocol shape is stable enough.
- [ ] Track any remaining open naming/vocabulary decisions after detailed spec updates.

## Next recommended steps

1. Have protocol worker compare Tandem protocol against Brainfile v2 plus local v3 direction.
2. Have CLI/TUI worker map Brainfile CLI/TUI features into keep/improve/omit/open categories.
3. Apply locked v0 CLI command scope to CLI/TUI docs.
4. Add protocol examples/fixtures only when the protocol shape is stable enough to make them useful.
5. Keep area READMEs, specs, and todos synchronized as implementation begins.
6. Consider dogfooding Tandem by creating a `.tandem/` workspace after the protocol is minimally stable.

## Open questions

- Should the monorepo use top-level `crates/` later, keep implementation under `tandem-tui/`, or use another minimal layout?
- When, if ever, should protocol schemas/fixtures be introduced?
- Should root planning docs remain Markdown, or migrate into Tandem documents once bootstrapped?
- Should `td` be reserved only for Pi/tool prefixes while `tdm` remains the user-facing CLI?
