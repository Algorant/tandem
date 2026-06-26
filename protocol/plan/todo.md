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
- [x] Captured Brainfile-inspired design mapping.
- [x] Drafted CLI surface using `tdm`.
- [x] Added `protocol/README.md` for protocol-area documentation.

## Current tasks

- [ ] Keep `protocol/README.md`, `plan/spec.md`, and `plan/todo.md` synchronized with parent docs.
- [ ] Audit live Brainfile protocol v2 against Tandem protocol draft and record keep/rename/improve/omit decisions without adding import requirements.
- [ ] Fold in `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md` review/completion/log direction.
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

1. Produce a Brainfile protocol parity matrix: keep, rename, improve, omit, open.
2. Reconcile local v3 proposal into the Tandem protocol lifecycle.
3. Create example Tandem workspace fixtures only after the protocol shape is stable enough.
4. Write JSON Schema or Rust struct sketches later, only when implementation needs them, for:
   - workspace config
   - work document
   - accord
   - review
   - event records
5. Define mutation semantics precisely:
   - add work
   - move state
   - update accord
   - request/accept review
   - complete/archive
   - reopen/restore
6. Define minimal-diff file editing requirements.
7. Update parent and CLI/TUI docs whenever protocol decisions affect them.
8. Decide implementation structure only when the data model is stable enough and the user approves.

## Acceptance criteria for protocol v0 draft

- [ ] A human can create/edit valid Tandem files by hand.
- [ ] An agent can read rules, claim work through an accord, deliver evidence, and request review.
- [ ] A tool can list active work without reading event history.
- [ ] A tool can show rich completed history from logs/events.
- [ ] Brainfile-inspired design differences are documented clearly.
- [ ] Unknown fields are preserved by compliant tooling.
