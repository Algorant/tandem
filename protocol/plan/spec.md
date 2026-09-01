# Tandem Protocol 0.3.0 Specification

**Status:** accepted normative specification  
**Date:** 2026-08-30

This document defines the only runtime protocol supported by Tandem 0.3.0. `protocol/` is normative; Rust in `tandem/src/protocol/` is executable semantics. No migration, conversion, compatibility reader, backup, or fallback is defined.

## Workspace

`.tandem/tandem.md` is YAML-frontmatter Markdown. Required fields are `protocolVersion: 0.3.0`, `title`, and `states`. The default states are `todo`, `in-progress`, and `validation`. Active records are in `.tandem/tasks/`; Decisions in `.tandem/decisions/`; Rules in `.tandem/rules/`; archived Tasks in `.tandem/logs/`; audit ledgers in `.tandem/events/<actor-id>.jsonl`. `.tandem/actor-id` is ignored checkout-local identity and is generated/owned by Tandem.

Discovery is repository-local and requires `.tandem/tandem.md`. A different protocol version is an operational error identifying detected and required versions.

## Task records

Task frontmatter requires `id`, `type: task`, `title`, `state` (active records), and `accord` (active records). Optional fields are `kind: epic`, `priority` (`low|medium|high|critical`), `effort` (`trivial|small|medium|large`), `tags`, `assignee`, `parentId`, `blockers`, `references`, `relatedFiles`, timestamps, and legacy inline `subtasks` preserved as inert checklist data.

Roles are resolved from documents, never inferred from ID shape:

- Epic: `kind: epic`, root-only, global `task-N` ID.
- Task: normal task, root or direct child of an Epic, global `task-N` ID.
- Subtask: normal task directly beneath a Task, immutable `task-N-M` ID, leaf.

Only these relationships exist: `epic-task` and `subtask`. Parent IDs resolve to active Tasks only. Epics cannot have parents, Subtasks cannot have children, arbitrary depth is invalid, and reparenting that changes role or invalidates an ID is rejected. Allocation scans active and archived records without reuse.

Papercuts are ordinary low-priority Tasks tagged `papercut`; no Papercut type or storage exists.

## Decisions

Decision records require `id`, `type: decision`, and `title`, and remain in `decisions/`. Optional metadata is `status` (`proposed|accepted|rejected|deprecated|superseded`), `deciders`, `supersedes`, `references`, `tags`, `createdAt`, `updatedAt`, and automatic `decidedAt` on acceptance/rejection. ADR Context, Decision, Consequences, Alternatives, and Supersession are Markdown body sections. Manual `date`, `supersededBy`, and prose metadata flags are not fields.

## Rules

Each Rule is one Markdown file in `rules/` with composite ID `<category>-N`, category `always|never|prefer|context`, optional `source`, `createdAt`, and `updatedAt`. Rule text is the Markdown body. Reclassification is delete-and-add. Missing sources are warnings.

## Accord

Active statuses are `ready`, `claimed`, `delivered`, `rework`, and `blocked`; `accepted` and `failed` occur only in Logs. Every active Task has at least one `accord.acceptance` criterion. Optional accord fields are `constraints`, planned `validation`, current `note`, and delivery data. Accord definition fields — `acceptance`, `constraints`, and planned `validation` — are durable: every status transition preserves them unchanged. Only an explicit `update` may rewrite them.

`claim` sets top-level `assignee`; `deliver` requires `summary` and one or more `evidence`; `rework`, `block`, `release`, and `fail` take one note; `resume` changes blocked to claimed; `release` clears assignee and returns to ready. `complete` atomically accepts delivered work and archives it. `fail` atomically archives failure.

`review <id> --criterion ... --note ...` is the only exceptional entry to `state: validation`. It records human escalation metadata without a review status field. Completion accepts; Accord rework returns to `in-progress`.

## Logs and events

Archive preserves the complete Task and Accord and adds only `archivedAt` and `resolution` (`outcome: completed|canceled|failed`, optional `note`, `reviewer`). Historical Logs may omit Accord. Delivery evidence is not copied into resolution metadata.

Every durable mutation emits exactly one structured event. Reads emit none. Each actor appends to its own JSONL file. The required envelope is `ts`, `seq`, `actor`, `event`, `id`, and event-specific `data`; bodies and a required free-text summary are forbidden in the envelope.

## Interface requirements

The canonical CLI uses clap derive and generated help. Its 23 leaves are:

```text
init
add task|decision
show
list
search
update
accord claim|deliver|rework|block|resume|release|fail
review
complete
cancel
rules list|add|edit|delete
tui
web
```

Global `-j/--json`, `-h/--help`, and `-V/--version` are accepted before or after commands. Help/version are human text and never discover a workspace. JSON success and failures are stdout-only envelopes; human results are stdout and warnings/errors stderr. Exit status is 0 success, 1 operational failure, 2 usage failure.

`list` and `search` use `--scope active|archived|all` and default to active. `update` infers type, cannot write Task state, assignee, or Accord status, replaces present repeated lists deterministically, leaves absent lists unchanged, and uses repeated `--clear` for removal. Human prose values accept leading hyphens; typed values remain strict.
