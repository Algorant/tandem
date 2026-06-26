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
- [x] Chose `accord` as the work-agreement object replacing Brainfile contract terminology.
- [x] Chose canonical workflow fields: `state` on documents and `states` in workspace config.
- [x] Chose default active states: `todo`, `in-progress`, and `review`.
- [x] Chose completion lifecycle: `todo → in-progress → review → complete/archive → logs`.
- [x] Captured separation between human workflow state, accord state, and review state.
- [x] Chose new task identity shape: `type: task` with sequential IDs such as `task-1`.
- [x] Chose first-class document types: `task` and `decision`.
- [x] Decided custom document types are config-only in v0, with no type-management CLI.
- [x] Chose accord statuses: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, and `blocked`.
- [x] Chose structured rule objects with `id`, `rule`, and optional `source`.
- [x] Decided `parentId`, `blockers`, and `references` may point to any Tandem document by ID.
- [x] Chose parent-based sequential subtask IDs such as `task-1-1`.
- [x] Decided completion warns about missing accepted review or accord but allows completion in v0.
- [x] Decided archived Markdown documents in `.tandem/logs/` are the source of truth for completed work.
- [x] Decided `.tandem/events.jsonl` enriches timeline/audit history.
- [x] Decided v0 validation/lint is built-in structural validation only.
- [x] Chose `protocolVersion: 0.1.0` for the first v0 draft.
- [x] Chose minimal audit-only event payloads requiring `ts`, `event`, `id`, and `summary`.
- [x] Chose strict-core-reference validation severity: unresolved `parentId`/`blockers` are errors; unresolved related `references` and rule sources are warnings.
- [x] Decided `type: decision` documents do not need a lifecycle field in v0.
- [x] Decided schemas and fixtures are not part of v0.
- [x] Captured Brainfile design mapping as reference only, with no required conversion command.
- [x] Drafted task document frontmatter model.
- [x] Drafted decision document frontmatter model.
- [x] Drafted accord lifecycle.
- [x] Drafted review model.
- [x] Drafted completion/logs/events model.
- [x] Drafted protocol-facing CLI surface using `tdm`.
- [x] Added `protocol/README.md` for protocol-area documentation.

## Current tasks

- [ ] Keep `protocol/README.md`, `plan/spec.md`, and `plan/todo.md` synchronized with parent docs.
- [ ] Tighten the v0 field reference for workspace config, task documents, decision documents, accords, reviews, completion metadata, events, and rules.
- [ ] Define mutation semantics precisely:
  - add task
  - add decision
  - move state
  - update accord
  - request/accept review
  - complete/archive
  - reopen/restore
- [ ] Define lifecycle event name catalog using the minimal audit-only event payload shape.
- [ ] Expand structural validation diagnostics using the strict-core-reference severity policy.
- [ ] Define completed-log document requirements and how events enrich log timelines.
- [ ] Define minimal-diff file editing requirements for compliant tools.

## Next recommended steps

1. Expand the locked v0 decisions into a concise field-by-field protocol reference.
2. Write mutation semantics for the core lifecycle operations.
3. Specify the structural validation diagnostic catalog from the locked warning/error policy.
4. Specify event names for task, accord, review, completion, and restore operations using the locked minimal audit event shape.
5. Add small inline examples for valid task and decision documents if the spec needs more clarity.
6. Update parent and CLI/TUI docs whenever protocol decisions affect them.
7. Keep schemas and fixtures out of v0; revisit only after CLI MVP/protocol stabilization and explicit approval.

## Acceptance criteria for protocol v0 draft

- [ ] A human can create/edit valid Tandem files by hand.
- [ ] An agent can read rules, claim work through an accord, deliver evidence, and request review.
- [ ] A tool can list active tasks without reading event history.
- [ ] A tool can browse first-class decision documents.
- [ ] A tool can show rich completed history from archived Markdown logs plus event timelines.
- [ ] Brainfile-inspired design differences are documented clearly without creating a conversion requirement.
- [ ] Unknown fields are preserved by compliant tooling.
