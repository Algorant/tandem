# Tandem Protocol Todo

Status: v0 draft accepted for implementation
Last updated: 2026-06-26

This todo tracks protocol-specific tasks. The current protocol draft lives in `protocol/plan/spec.md`.

## Accomplished

- [x] Captured Brainfile-inspired file-based model.
- [x] Chose working protocol/product name: **Tandem**.
- [x] Chose protocol data layout:
  - `.tandem/tandem.md`
  - `.tandem/board/`
  - `.tandem/logs/`
  - `.tandem/events/<actor_id>.jsonl` per-actor logs (with legacy `.tandem/events.jsonl` reads during transition)
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
- [x] Decided archived Markdown documents in `.tandem/logs/` are the source of truth for completed history.
- [x] Decided per-actor event logs under `.tandem/events/<actor_id>.jsonl` enrich timeline/audit history while avoiding a shared Git append hotspot; legacy `.tandem/events.jsonl` remains readable during transition.
- [x] Decided v0 validation/lint is built-in structural validation only.
- [x] Chose `protocolVersion: 0.1.0` for the first v0 draft.
- [x] Chose minimal audit-only event payloads requiring `ts`, `event`, `id`, `summary`, `actor`, and `seq`, with event identity `<actor>:<seq>` and optional cosmetic `actorName`.
- [x] Chose strict-core-reference validation severity: unresolved `parentId`/`blockers` are errors; unresolved related `references` and rule sources are warnings.
- [x] Decided `type: decision` documents do not need a lifecycle field in v0.
- [x] Decided schemas and fixtures are not part of v0.
- [x] Captured Brainfile design mapping as reference only, with no required conversion command.
- [x] Drafted task document frontmatter model.
- [x] Drafted decision document frontmatter model.
- [x] Drafted accord lifecycle.
- [x] Drafted review model.
- [x] Drafted completion/logs/events model.
- [x] Drafted protocol-facing CLI surface using `tandem`.
- [x] Added `protocol/README.md` for protocol-area documentation.
- [x] Added implementation-facing v0 field reference for workspace config, task documents, decision documents, accords, reviews, completion metadata, logs, and rules.
- [x] Added minimal audit event envelope, actor identity/sequence rules, legacy-read behavior, and event name catalog.
- [x] Added validation diagnostics with error/warning categories and examples.
- [x] Defined completed-log document expectations.
- [x] Defined mutation semantics for adding tasks/decisions, moving state, updating accords, review decisions, complete/archive, and post-v0 restore/reopen boundaries.
- [x] Accepted protocol v0 draft for implementation.

## Current tasks

- [ ] Tighten examples if implementation discovers ambiguous field behavior.

## Next recommended steps

1. Implement the first `tandem` CLI slice in `../tandem/`: `init`, `list`, and `show`.
2. Tighten protocol examples only when implementation discovers ambiguous behavior.
3. Keep schemas, fixtures, and protocol implementation layout out of v0 unless explicitly approved.

## Acceptance criteria for protocol v0 draft

- [x] A human can create/edit valid Tandem files by hand.
- [x] An agent can read rules, claim work through an accord, deliver evidence, and request review.
- [x] A tool can list active tasks without reading event history.
- [x] A tool can browse first-class decision documents.
- [x] A tool can show rich completed history from archived Markdown logs plus event timelines.
- [x] Brainfile-inspired design differences are documented clearly without creating a conversion requirement.
- [x] Unknown fields are preserved by compliant tooling.
