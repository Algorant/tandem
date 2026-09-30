# Tandem Protocol 0.4.0

This directory is the normative specification for Tandem's local-first coordination format. The executable implementation is `tandem/src/protocol/`; filesystem discovery and persistence belong to `tandem/src/project/`.

## Workspace layout

A 0.4.0 workspace contains:

```text
.tandem/
  tandem.md
  tasks/       # active Task and Epic Markdown records
  decisions/   # durable Decision Markdown records
  rules/       # one Rule per Markdown file
  logs/        # archived Task records
  events/      # one JSONL ledger per actor
```

In a Git repository the board is the `.tandem/` folder of the main worktree, ignored by the source branch and synchronized through the repository's `tandem` branch (see [Sync](#sync)). Every linked worktree uses the main worktree's board. A board outside Git is a plain local folder.

The workspace frontmatter must contain `protocolVersion: 0.4.0`, a permanent `workspaceId` (random UUID), `title`, and the active `states` (`todo`, `in-progress`, `validation`). A migrated board also records `migratedFrom`, the source commit whose 0.3.0 board it came from. Another protocol version fails clearly with both detected and required versions; a 0.3.0 board is converted once with `tandem migrate`.

Actor identity is checkout-local: `<git-dir>/tandem-actor-id` for the current checkout (a linked worktree has its own Git directory), or `.tandem/actor-id` for a board outside Git. It is never inside the synced board and never committed.

## Documents

Only `task` and `decision` are first-class documents. Unknown fields and Markdown bodies are preserved.

Every Task, Subtask, Epic, Decision, and Rule has a permanent `uid` (random UUID) written at creation. The `uid` identifies the record across machines and never changes. The human `id` is sequential. A record created on a board that syncs through a remote first receives a provisional ID `<prefix>-new-<first 8 hex of uid>` (for example `task-new-3f9a2c1d`, `decision-new-…`, `always-new-…`); publication assigns the next sequential ID against the shared board, renames the file, and rewrites every whole-token occurrence in records and event ledgers. The shared `tandem` branch never contains a provisional ID, and a published ID never changes. A lookup by an outdated provisional ID resolves through the uid prefix; an ambiguous prefix is an error. A board without a remote allocates sequential IDs directly.

Tasks use immutable global `task-N` IDs for root Tasks, Epics, and direct Epic children. A normal Task directly beneath a normal Task is a leaf Subtask with immutable `task-N-M` ID. A provisional `task-new-<hex>` ID is valid in every role until publication. Epics (`kind: epic`) are root-only. Subtasks cannot have children, and reparenting may not change role or invalidate the ID. `parentId` is task-only and resolves only Epic → Task or Task → Subtask relationships. Decisions are linked with `references`, never hierarchy.

Active Tasks have `state` and a mandatory `accord` with at least one `acceptance` criterion. State is exactly `todo`, `in-progress`, or `validation`. Papercuts are ordinary low-priority Tasks tagged `papercut`.

Decision metadata includes `status` (`proposed`, `accepted`, `rejected`, `deprecated`, `superseded`), automatic `createdAt`/`updatedAt`, automatic `decidedAt` on acceptance or rejection, `deciders`, `supersedes`, `references`, and `tags`. ADR prose belongs in the Markdown body. There are no manual `date`, `supersededBy`, or prose metadata flags.

`decidedAt` records the latest actual transition into `accepted` or `rejected`, including creation directly in either status. It is retained when the decision later moves to `proposed`, `deprecated`, or `superseded`; repeating an unchanged status is a no-op and never backfills a historical record. Common `update` resolves the document's actual type and edits Decision metadata (`title`, `body`, `status`, `deciders`, `supersedes`, `references`, `relatedFiles`, `tags`) without rewriting unrelated fields or the Markdown body.

`references` accepts document IDs and absolute `http(s)` URLs. Document IDs resolve against the workspace, and an unresolved ID is a warning; absolute URLs are opaque loose links that Tandem never fetches, never rewrites, and never warns about. Repository paths are path metadata for `relatedFiles`, which is not validated. Reference warnings cover active Board records (Tasks and Decisions); archived Logs are immutable history and never emit missing-target warnings.

Rules use composite IDs such as `always-12`, a category (`always`, `never`, `prefer`, or `context`), optional `source`, timestamps, and rule text as the Markdown body. Reclassification is delete-and-add.

## Accord and archive

Accord statuses are `ready`, `claimed`, `delivered`, `rework`, `blocked`, and terminal `accepted` or `failed` in Logs. `ready` requires acceptance criteria. `claim` sets top-level `assignee`; `release` clears it and returns to `ready`; `deliver` requires a summary and at least one evidence entry containing non-whitespace text; `resume` changes blocked to claimed. `complete` atomically accepts delivered work and archives it. `fail` atomically archives failed work. `cancel` archives canceled work. Archived records retain the full Task and Accord plus `archivedAt` and minimal `resolution: { outcome, note, reviewer }`; delivery evidence is not duplicated.

Completing a Task warns when its own Accord is neither delivered nor accepted. One narrow exception exists: a grouping Epic (`kind: epic`) whose resolved child hierarchy is nonempty and whose every descendant Task or Subtask is archived with an explicit canonical `resolution.outcome: completed` is eligible to close without that warning. Eligibility only suppresses the missing-delivery warning: for an otherwise-undelivered eligible Epic, completion adds no synthetic Accord delivery or acceptance and copies no child evidence, while normal archive and sync behavior still runs and an explicitly delivered parent still becomes accepted under the ordinary rule. Every other completion check is unchanged: active Board descendants still block closure, unresolved blockers and structural validation still fail, and an empty Epic and any absent, legacy-only, unknown, canceled, or failed descendant outcome retain the ordinary warning.

Archive outcome and delivery status are distinct. `resolution.outcome` records how an archived record ended (`completed`, `canceled`, or `failed`) and is not an Accord status; a missing or legacy-only `completion.outcome` record is not positive evidence of completion and does not qualify for child-based closure.

Validation is exceptional human escalation: `review <id>` requires the exact unresolved `criterion` and a `note`, enters `state: validation`, and may include a reviewer. Completion accepts; Accord rework returns to `in-progress`. Review status is not stored.

`accord release <id> --note <text>` returns work to `ready`, resets workflow `state` to `todo`, and clears its assignee so the Task is claimable again. It accepts an optional `--disposition reassign|discarded`, defaulting to `reassign`; disposition is event data on `accord.released`, not an Accord status. Read projections derive `attemptCount` from `accord.claimed` events, `reworkCount` from `accord.rework` events, and `discardedCount` from released events whose disposition is `discarded`. Counts cover every attempt for the Task, including archived Tasks, and are not editable document fields. Adapters should record post-delivery corrections with `accord rework` rather than out-of-band messages so the counts remain accurate.

## Events

Every durable mutation writes one event to `.tandem/events/<actor-id>.jsonl`; reads never write. The required envelope is:

```json
{"ts":"...","seq":4,"actor":"...","event":"task.updated","id":"task-12","data":{"fields":["tags"]}}
```

Required fields are `ts`, `seq`, `actor`, `event`, `id`, and structured event-specific `data`. Full bodies are never copied into events.

## Sync

A Git-backed board synchronizes through the repository's `tandem` branch on its remote (`git config tandem.remote`, else `origin`, else the only remote). Source commits never contain the board, so board changes never stage, commit, or block source-branch operations.

Local state lives in the Git directory: `refs/tandem/pending` (safety copy of the latest local snapshot), `refs/tandem/base` (the last `tandem` commit merged), `refs/tandem/remote` (last fetched tip), and private files under `<git-common-dir>/tandem/` (lock, index, open conflicts, status).

One sync, under the board lock:

1. Restore board files missing without a Tandem command (after `git clean` or an old checkout) from the safety copy, snapshot the board, and update the safety copy. Only board content syncs: `tandem.md`, top-level `*.toml`, `tasks|decisions|rules|logs/*.md`, and `events/*.jsonl`.
2. Hold back local changes that cannot be published: unparsable files, missing `uid`, `id` not matching the file name, a changed `uid` or published ID, a changed `protocolVersion` or `workspaceId`, and records with an open conflict.
3. Merge base, local, and remote record by record, matching records by `uid` (an archive move is the same record). Changes on one side apply; different fields combine; set-like lists (`tags`, `references`, `blockers`, `relatedFiles`, `supersedes`, `deciders`, `filesChanged`) merge as sets; `updatedAt` takes the later value; Markdown bodies merge three-way when they do not overlap; event ledgers merge only when one extends the other. Anything else changed differently on both sides, and archive on one side with an edit on the other, is a conflict: that record keeps the remote version, both versions are preserved locally, and everything else continues to sync. Time never picks a winner and conflict markers are never written.
4. Number provisional records and validate the complete merged board. A local change that would make it invalid is held back.
5. Push without force. A rejected push fetches and merges again. Only after the remote accepts the commit is the result written into the board and `base` advanced.

`tandem sync resolve <id> --keep local|remote|edited` settles a conflict; changes to a conflicted record are refused until then. Offline, changes stay saved and reported as pending. When the `.tandem/tandem.md` present comes from an older commit (protocol 0.3.0 while this clone has sync state), the board is shown read-only with a warning and does not sync; returning to a current commit restores it.

Mutations publish immediately. Reads refresh first when the last fetch is older than 60 seconds. The TUI and web interface sync in the background while open. Nothing runs while Tandem is closed.

## Migration

`tandem migrate` converts a 0.3.0 board that source commits track: it requires a remote without a `tandem` branch, a branch not behind its upstream, and no staged changes; adds a `uid` to every record and `workspaceId`/`migratedFrom` to `tandem.md`; publishes the board as the root commit of the `tandem` branch; moves the checkout's actor identity into its Git directory; and creates one source commit that stops tracking `.tandem/` and ignores it. `--dry-run` reports without changing anything.

On another machine with unpushed or uncommitted 0.3.0 board changes, `tandem migrate --adopt` (before pulling the migration commit) merges them into the shared board from the common legacy base, gives records that were never published provisional IDs so they are renumbered, stores the result in the safety copy, and restores the tracked legacy files so `git pull` can remove them. A machine without local board changes just pulls; its first Tandem command downloads the board.

## CLI contract

The Rust CLI is clap-derived. The exact command tree is documented by generated help and has 28 leaves: `init`; `add task|decision`; `show`; `assignment`; `list`; `search`; `update`; `accord claim|deliver|rework|block|resume|release|fail`; `review`; `complete`; `cancel`; `sync`; `sync status|resolve`; `migrate`; `rules list|add|edit|delete`; `tui`; and `web`. Global `-j/--json`, `-h/--help`, and `-V/--version` work before or after subcommands. JSON success and operational/usage errors are stdout-only envelopes. Human results use stdout and warnings/errors use stderr. Exit codes are 0 success, 1 operational failure, and 2 usage failure. Every mutation's JSON result includes `data.sync`: `{"status":"synced|pending|local-only|not-git","message":...,"renamed":{"<provisional>":"<id>"},"conflicts":[{"id","reason"}],"held":[{"path","reason"}]}` and reports the record's final ID. A sync that cannot complete never undoes a saved change and never fails the command; the result is `pending`.

`assignment <task-id> --json` returns the complete current Task and direct milestone definition with an opaque freshness token and derived blocker readiness; assignment nodes also include derived `attemptCount`, `reworkCount`, and `discardedCount`; see [`assignment.md`](assignment.md). A planned validation beginning with `$ ` is a runnable command: after removing the prefix and leading whitespace, the command runs from the Task repository root and exit 0 passes. Other planned validations are manual checks. Assignment JSON classifies each item as `{ "kind": "command" | "manual", "text": "..." }` and strips `$ ` from command text; `show --json` preserves the raw validation strings. Adapters should require captured command output for command entries and may refuse integration on a non-zero exit. Tandem does not execute these commands, and Tasks with no command entries are unaffected. `list` and `search` support `--scope active|archived|all`, defaulting to active. `update` never mutates state, assignee, or Accord status. Repeated list values replace the complete list; absent values remain unchanged; `--clear` removes lists and optional scalars. Human prose accepts leading hyphens while typed IDs, enums, and numbers remain strict.

There is no `upgrade`, `move`, `version`, or `checkpoint` command, `log`, `decision`, or `papercut` command family, compatibility parser, or fallback. `migrate` is the one-time 0.3.0 conversion.
