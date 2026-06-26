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

## Current tasks

- [ ] Keep parent project plan aligned with `protocol/README.md`, `protocol/plan/spec.md`, `tandem-tui/README.md`, and `tandem-tui/plan/spec.md`.
- [ ] Enforce no-drift documentation updates whenever naming, scope, architecture, lifecycle, or workflow decisions change.
- [ ] Decide whether root `plan/todo.md` remains the canonical high-level tracker or whether Tandem should dogfood its own protocol once bootstrapped.
- [ ] Define initial Rust workspace layout once implementation begins.
- [ ] Decide initial crate names and binary names.
- [ ] Decide whether `tdm tui` and `tdm-tui` should both exist.
- [ ] Create initial examples/fixtures directory.
- [ ] Track open naming/vocabulary decisions.

## Next recommended steps

1. Add protocol examples/fixtures under `protocol/examples/` or `protocol/fixtures/`.
2. Start schema sketching from `protocol/plan/spec.md`.
3. Create a Rust workspace skeleton when ready:
   - `crates/tandem-core`
   - `crates/tdm-cli`
   - `tandem-tui` or `crates/tandem-tui`
4. Add CI once there is code to validate.
5. Keep area READMEs, specs, and todos synchronized as implementation begins.
6. Consider dogfooding Tandem by creating a `.tandem/` workspace after the protocol is minimally stable.

## Open questions

- Should the monorepo use top-level `crates/` later, or keep `tandem-tui/` at the root permanently?
- Should protocol schemas live in `protocol/schema/` or inside a Rust core crate?
- Should root planning docs remain Markdown, or migrate into Tandem documents once bootstrapped?
- Should `td` be reserved only for Pi/tool prefixes while `tdm` remains the user-facing CLI?
