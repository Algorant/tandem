# Tandem Protocol

This directory contains the normative Tandem protocol specification.

The Markdown here is the source of truth for Tandem format and semantics. The
executable Rust implementation lives in `../tandem/src/protocol/`; it implements
these requirements and is not a second specification. Concrete project discovery,
raw-source preservation, locking, and atomic filesystem writes belong to
`../tandem/src/project/`, not to the normative document model.

The protocol defines the local-first file format for human/agent coordination. It is inspired by Brainfile's useful shape, adapted into Tandem terminology, and extended with the local v3 direction around review, complete/archive, and first-class logs. It has no v0 Brainfile import/migration requirement.

## Scope

The protocol area owns:

- `.tandem/` workspace layout
- `.tandem/tandem.md` workspace config shape
- active work documents in `.tandem/board/`
- completed and canceled work-history documents in `.tandem/logs/`
- tracked per-actor `.tandem/events/<actor_id>.jsonl` lifecycle ledgers, with ignored checkout/worktree-local `.tandem/actor-id` identity and legacy `.tandem/events.jsonl` reads during transition
- `accord` work-agreement model
- review and completion semantics
- Brainfile-inspired protocol parity decisions
- local v3 proposal reconciliation from `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md`
- post-v0 schema/fixture definitions only if explicitly useful later

The protocol area does **not** own TUI rendering details. TUI design belongs in `../tandem/`.

## Current status

Protocol `0.2.0` is implemented in the single Rust binary crate under
`../tandem/`. No separate protocol crate, schemas, or fixtures exist, and
schemas/fixtures are not part of v0. Protocol docs should change only for
implementation feedback, bug fixes, or explicit product decisions.

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
- Logs are first-class terminal work history: missing `completion.outcome` means completed, while reasoned cancellation uses `completion.outcome: canceled`.
- Match Brainfile's basic protocol feature shape unless Tandem intentionally improves or omits something.


## Locked v0 protocol decisions

- Protocol version: `0.2.0`. Tandem refuses ordinary project operations on discovered `0.1.0` workspaces until the user explicitly runs `tandem upgrade`; help and version remain available.
- Canonical workflow field: `state`; default states: `todo`, `in-progress`, `validation` (with legacy `review` reads tolerated).
- New work items use `type: task`; the canonical shape is `task-10` Epic → `task-11` global Task → `task-11-1` parent-derived leaf Subtask. Epics and Tasks—including direct Epic Tasks—use global `task-N` IDs. Only a Subtask directly beneath a Task uses `task-N-M`.
- First-class document types: `task` and `decision`; decision docs are ADR-compatible durable records and do not need a lifecycle field. Existing custom declarations/documents are deprecated read-only content: upgrade preserves them for list/show/search, but Tandem neither creates nor mutates them.
- Epic, Task, and Subtask are derived roles over normal task documents. An Epic is `type: task` plus `kind: epic`; a Task is normal and root-level, generic-parented, or directly Epic-parented; a Subtask is normal and directly parented by a Task. Classification resolves documents and never uses ID shape.
- Direct Epic children use `epic-task`; Task children use `subtask`; decision/custom-document links use generic `parent`. Generic-parent Tasks may have Subtasks.
- Strict validation rejects a parented Epic, a child beneath a Subtask, any role/ID mismatch, and role-changing or ID-invalidating reparenting.
- `parentId` remains canonical for hierarchy, while the resolved role constrains ID shape: Epics/Tasks require global `task-N`; Subtasks require `task-N-M`. Direct Epic Tasks with hierarchical IDs and Subtasks with global IDs are invalid.
- Decision-7 fully supersedes decision-4 with no compatibility exception. Global and per-Task suffix allocation both scan active board documents and completed logs without reuse.
- Inline `subtasks:` checklist items are legacy and deprecated for new work. Existing entries remain readable, validatable, and preservable; new lifecycle-bearing checklist work uses first-class Subtask documents.
- Epics retain normal task lifecycle and have no separate type, ID namespace, command family, or lifecycle. Epics are not delegated; a delegated Task's Subtask documents are Worker A's `pi-todos` execution checklist and are not independently delegated.
- Accord statuses: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`.
- Rules are structured objects. References can point to any Tandem document by ID.
- Completion always warns but allows completion unless structural validation blocks it. Legacy project-level completion-policy settings are preserved, deprecated, and ignored.
- Completed logs are archived markdown docs in `.tandem/logs/`; minimal audit-only events live in tracked per-actor `.tandem/events/<actor_id>.jsonl` logs, while legacy `.tandem/events.jsonl` remains readable during transition. Tandem persists the automatic actor UUID in ignored `.tandem/actor-id` per independent checkout or linked worktree.
- Validation is built-in structural validation only, with strict structure/core refs, hierarchy roles, and ID grammar: unresolved `parentId`/`blockers`, parented Epics, children beneath Subtasks, role/ID mismatches, role-changing reparenting, and invalid optional `priority` (`low|medium|high|critical`) or `effort` (`trivial|small|medium|large`) values are errors; unresolved related `references` are warnings.
- No Brainfile import/migration command is required in v0.
