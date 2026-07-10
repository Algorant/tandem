# Tandem Protocol

This directory contains the Tandem protocol/spec planning.

The protocol defines the local-first file format for human/agent coordination. It is inspired by Brainfile's useful shape, adapted into Tandem terminology, and extended with the local v3 direction around review, complete/archive, and first-class logs. It has no v0 Brainfile import/migration requirement.

## Scope

The protocol area owns:

- `.tandem/` workspace layout
- `.tandem/tandem.md` workspace config shape
- active work documents in `.tandem/board/`
- completed work documents in `.tandem/logs/`
- per-actor `.tandem/events/<actor_id>.jsonl` lifecycle ledgers, with legacy `.tandem/events.jsonl` reads during transition
- `accord` work-agreement model
- review and completion semantics
- Brainfile-inspired protocol parity decisions
- local v3 proposal reconciliation from `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md`
- post-v0 schema/fixture definitions only if explicitly useful later

The protocol area does **not** own TUI rendering details. TUI design belongs in `../tandem/`.

## Current status

Protocol v0 draft is accepted for implementation. No protocol crate, schemas, or fixtures exist, and schemas/fixtures are not part of v0. Implementation begins in `../tandem/`; protocol docs should change only for implementation feedback, bug fixes, or explicit product decisions.

## Documentation

- `plan/spec.md` — protocol draft
- `plan/todo.md` — protocol task tracker
- `../README.md` — parent project overview
- `../plan/spec.md` — parent project plan
- `../plan/todo.md` — parent project todo
- `../AGENTS.md` — agent guidance

## Key current decisions

- Product/protocol name: **Tandem**
- CLI binary: `tandem`
- Protocol data directory: `.tandem/`
- Config file: `.tandem/tandem.md`
- Work agreement object: `accord`
- Completion is an action/archive transition, not a default `done` column.
- Human workflow state, accord state, and review state are separate.
- Logs are first-class completed-work history.
- Match Brainfile's basic protocol feature shape unless Tandem intentionally improves or omits something.


## Locked v0 protocol decisions

- Protocol version: `0.1.0` for the first v0 draft.
- Canonical workflow field: `state`; default states: `todo`, `in-progress`, `validation` (with legacy `review` reads tolerated).
- New work items use `type: task`; default allocation uses flat sequential IDs such as `task-1`.
- First-class document types: `task` and `decision`; decision docs are ADR-compatible durable records, do not need a lifecycle field in v0, and should not be split into a separate ADR type; custom types are config-only.
- A first-class subtask is a normal `type: task` document linked to another task with `parentId`; it keeps normal task workflow, ownership, accord, review, and completion behavior, with no new type or relationship field.
- Parent-derived hierarchical IDs such as `task-100-1` are allowed and recommended when useful, but optional; `parentId`, not the ID, defines hierarchy.
- Inline `subtasks:` checklist items are legacy and deprecated for new work. Existing entries remain readable, validatable, and preservable; new trackable work should use child task documents.
- Epics are ordinary `type: task` documents with `kind: epic` for broad outcome grouping; they use the same general `parentId` hierarchy, ordinary tasks may also parent children, loose related context uses `references`, and v0 has no separate epic type, ID namespace, command family, or lifecycle.
- Accord statuses: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`.
- Rules are structured objects. References can point to any Tandem document by ID.
- Completion warns but allows completion in v0.
- Completed logs are archived markdown docs in `.tandem/logs/`; minimal audit-only events live in per-actor `.tandem/events/<actor_id>.jsonl` logs, while legacy `.tandem/events.jsonl` remains readable during transition.
- Validation is built-in structural validation only, with strict structure/core refs: unresolved `parentId`/`blockers` are errors; unresolved related `references` are warnings.
- No Brainfile import/migration command is required in v0.
