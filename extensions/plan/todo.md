# Tandem Extensions Todo

Status: active implementation  
Last updated: 2026-06-28

This todo tracks agent/editor integration work under `extensions/`.

## Accomplished

- [x] Added `extensions/` as the third major child area for Tandem integrations.
- [x] Documented the adapter principle: extensions call `tdm`; protocol behavior stays in `protocol/` and `tandem-tui/`.
- [x] Created the initial `pi-tandem/` Pi extension scaffold.
- [x] Implemented a CLI-backed `pi-tandem` MVP with `tdm_*` tools and `/tandem` diagnostics.
- [x] Added a local smoke test that exercises the wrapper command mappings through a temporary `.tandem` workspace.

## Current tasks

- [ ] Keep extension docs synchronized with parent docs and affected integration docs.
- [ ] Test `pi-tandem` as a project-local Pi extension through `pi -e` or `.pi/extensions/` after review.
- [ ] Capture review feedback before promoting any extension into canonical global Pi config.

## Next recommended steps

1. Run project-local Pi smoke testing for `pi-tandem` in a real Pi session.
2. Add small renderers or UI polish only if raw text output proves hard to use.
3. Promote `pi-tandem` to shared Pi config in a separate reviewed task if the local adapter is accepted.

## Open questions

- Should future integrations live under `extensions/<target>/` with the same README/spec/todo standard, or should tiny integrations be allowed as single files after review?
- Should `tdm` eventually expose structured mutation output so adapters can return richer details without parsing human text?
