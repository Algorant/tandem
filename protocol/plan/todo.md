# Tandem Protocol Todo

Status: active planning  
Last updated: 2026-06-26

This todo tracks protocol-specific work. The current protocol draft lives in `protocol/plan/spec.md`.

## Accomplished

- [x] Captured Brainfile-inspired file-based model.
- [x] Chose working protocol/product name: **Tandem**.
- [x] Chose protocol data layout:
  - `.tandem/tandem.md`
  - `.tandem/board/`
  - `.tandem/logs/`
  - `.tandem/events.jsonl`
- [x] Chose `accord` as the work-agreement object replacing Brainfile `contract`.
- [x] Defined initial human workflow lifecycle:
  - `backlog/todo → active/in-progress → review → complete/archive → logs`
- [x] Captured separation between human workflow state, accord state, and review state.
- [x] Drafted work document frontmatter model.
- [x] Drafted accord lifecycle.
- [x] Drafted review model.
- [x] Drafted completion/logs/event-ledger model.
- [x] Drafted Brainfile import compatibility mapping.
- [x] Drafted CLI surface using `tdm`.
- [x] Added `protocol/README.md` for protocol-area documentation.

## Current tasks

- [ ] Keep `protocol/README.md`, `plan/spec.md`, and `plan/todo.md` synchronized with parent docs.
- [ ] Tighten vocabulary around `state`, `review`, `completion`, and `accord`.
- [ ] Decide default states: `todo/active/review` vs `backlog/todo/active/review`.
- [ ] Decide whether completion requires accepted review by default.
- [ ] Decide whether completion requires accepted accord when an accord exists.
- [ ] Decide ID strategy:
  - ULID-based IDs
  - slug + ULID filenames
  - sequential IDs with conflict mitigation
- [ ] Decide event ledger name: `events.jsonl`, `ledger.jsonl`, or `history.jsonl`.
- [ ] Define how logs are displayed: archived Markdown, event ledger, or merged view.
- [ ] Define protocol versioning and schema URLs.
- [ ] Define strictness and extension policy for v0.

## Next recommended steps

1. Create example Tandem workspace fixtures.
2. Write JSON Schema or Rust struct sketches for:
   - workspace config
   - work document
   - accord
   - review
   - event records
3. Define mutation semantics precisely:
   - add work
   - move state
   - update accord
   - request/accept review
   - complete/archive
   - reopen/restore
4. Define minimal-diff file editing requirements.
5. Draft Brainfile v2 importer behavior and migration report format.
6. Update parent and TUI docs whenever protocol decisions affect them.
7. Start `tandem-core` once the data model is stable enough.

## Acceptance criteria for protocol v0 draft

- [ ] A human can create/edit valid Tandem files by hand.
- [ ] An agent can read rules, claim work through an accord, deliver evidence, and request review.
- [ ] A tool can list active work without reading event history.
- [ ] A tool can show rich completed history from logs/events.
- [ ] Brainfile v2 boards can be imported with minimal data loss.
- [ ] Unknown fields are preserved by compliant tooling.
