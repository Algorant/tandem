# Tandem Protocol 0.3.0

This directory is the normative specification for Tandem's local-first coordination format. The executable implementation is `tandem/src/protocol/`; filesystem discovery and persistence belong to `tandem/src/project/`.

## Workspace layout

A 0.3.0 workspace contains:

```text
.tandem/
  tandem.md
  tasks/       # active Task and Epic Markdown records
  decisions/   # durable Decision Markdown records
  rules/       # one Rule per Markdown file
  logs/        # archived Task records
  events/      # one JSONL ledger per actor
```

`.tandem/actor-id` is checkout-local ignored runtime identity. There is no Board directory, Papercut directory, shared event ledger, migration reader, upgrade command, or compatibility path.

The workspace frontmatter must contain `protocolVersion: 0.3.0`, `title`, and the active `states` (`todo`, `in-progress`, `validation`). An encountered other protocol version fails clearly with both detected and required versions.

## Documents

Only `task` and `decision` are first-class documents. Unknown fields and Markdown bodies are preserved.

Tasks use immutable global `task-N` IDs for root Tasks, Epics, and direct Epic children. A normal Task directly beneath a normal Task is a leaf Subtask with immutable `task-N-M` ID. Epics (`kind: epic`) are root-only. Subtasks cannot have children, and reparenting may not change role or invalidate the ID. `parentId` is task-only and resolves only Epic → Task or Task → Subtask relationships. Decisions are linked with `references`, never hierarchy.

Active Tasks have `state` and a mandatory `accord` with at least one `acceptance` criterion. State is exactly `todo`, `in-progress`, or `validation`. Papercuts are ordinary low-priority Tasks tagged `papercut`.

Decision metadata includes `status` (`proposed`, `accepted`, `rejected`, `deprecated`, `superseded`), automatic `createdAt`/`updatedAt`, automatic `decidedAt` on acceptance or rejection, `deciders`, `supersedes`, `references`, and `tags`. ADR prose belongs in the Markdown body. There are no manual `date`, `supersededBy`, or prose metadata flags.

Rules use composite IDs such as `always-12`, a category (`always`, `never`, `prefer`, or `context`), optional `source`, timestamps, and rule text as the Markdown body. Reclassification is delete-and-add.

## Accord and archive

Accord statuses are `ready`, `claimed`, `delivered`, `rework`, `blocked`, and terminal `accepted` or `failed` in Logs. `ready` requires acceptance criteria. `claim` sets top-level `assignee`; `release` clears it and returns to `ready`; `deliver` requires a summary and at least one evidence entry containing non-whitespace text; `resume` changes blocked to claimed. `complete` atomically accepts delivered work and archives it. `fail` atomically archives failed work. `cancel` archives canceled work. Archived records retain the full Task and Accord plus `archivedAt` and minimal `resolution: { outcome, note, reviewer }`; delivery evidence is not duplicated.

Validation is exceptional human escalation: `review <id>` requires the exact unresolved `criterion` and a `note`, enters `state: validation`, and may include a reviewer. Completion accepts; Accord rework returns to `in-progress`. Review status is not stored.

`accord release <id> --note <text>` returns work to `ready` and clears its assignee. It accepts an optional `--disposition reassign|discarded`, defaulting to `reassign`; disposition is event data on `accord.released`, not an Accord status. Read projections derive `attemptCount` from `accord.claimed` events, `reworkCount` from `accord.rework` events, and `discardedCount` from released events whose disposition is `discarded`. Counts cover every attempt for the Task, including archived Tasks, and are not editable document fields. Adapters should record post-delivery corrections with `accord rework` rather than out-of-band messages so the counts remain accurate.

## Events

Every durable mutation writes one event to `.tandem/events/<actor-id>.jsonl`; reads never write. The required envelope is:

```json
{"ts":"...","seq":4,"actor":"...","event":"task.updated","id":"task-12","data":{"fields":["tags"]}}
```

Required fields are `ts`, `seq`, `actor`, `event`, `id`, and structured event-specific `data`. Full bodies are never copied into events.

## Native Git checkpoints

The native Rust application writes records and events immediately. Root Tasks
and direct Tasks beneath an Epic are assignments: their claim/start,
delivery, block/pause, resume/rework/release, terminal failure or cancel,
completion, and exceptional review/validation actions stage and commit the
owning `.tandem/` path. Epics are grouping records and Subtasks are assignment
milestones: their lifecycle writes return `batched` and remain durable without
creating a commit until an assignment boundary captures them. Intermediate
metadata and progress writes do not commit. Checkpoints use the fixed subject
`chore(tandem): checkpoint metadata`, never amend, and serialize through a
lock in Git's common directory so linked worktrees share one boundary. Git
checkpoint failure is returned separately from the successful record result;
there is no fallback or force path. See
[`plan/task-14-pi-handoff.md`](../plan/task-14-pi-handoff.md) for the consumer
JSON contract and cutover sequence.

## CLI contract

The Rust CLI is clap-derived. The exact command tree is documented by generated help and has 24 leaves: `init`; `add task|decision`; `show`; `assignment`; `list`; `search`; `update`; `accord claim|deliver|rework|block|resume|release|fail`; `review`; `complete`; `cancel`; `rules list|add|edit|delete`; `tui`; and `web`. Global `-j/--json`, `-h/--help`, and `-V/--version` work before or after subcommands. JSON success and operational/usage errors are stdout-only envelopes. Human results use stdout and warnings/errors use stderr. Exit codes are 0 success, 1 operational failure, and 2 usage failure.

`assignment <task-id> --json` returns the complete current Task and direct milestone definition with an opaque freshness token and derived blocker readiness; assignment nodes also include derived `attemptCount`, `reworkCount`, and `discardedCount`; see [`assignment.md`](assignment.md). A planned validation beginning with `$ ` is a runnable command: the remainder runs from the Task repository root and exit 0 passes. Other planned validations are manual checks. Assignment JSON classifies each item as `{ "kind": "command" | "manual", "text": "..." }` and strips `$ ` from command text; `show --json` preserves the raw validation strings. Adapters should require captured command output for command entries and may refuse integration on a non-zero exit. Tandem does not execute these commands, and Tasks with no command entries are unaffected. `list` and `search` support `--scope active|archived|all`, defaulting to active. `update` never mutates state, assignee, or Accord status. Repeated list values replace the complete list; absent values remain unchanged; `--clear` removes lists and optional scalars. Human prose accepts leading hyphens while typed IDs, enums, and numbers remain strict.

There is no `upgrade`, `migrate`, `move`, `version` command, `log`, `decision`, or `papercut` command family, compatibility parser, fallback, or adapter implementation in this cutover.
