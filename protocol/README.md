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

`decidedAt` records the latest actual transition into `accepted` or `rejected`, including creation directly in either status. It is retained when the decision later moves to `proposed`, `deprecated`, or `superseded`; repeating an unchanged status is a no-op and never backfills a historical record. Common `update` resolves the document's actual type and edits Decision metadata (`title`, `body`, `status`, `deciders`, `supersedes`, `references`, `relatedFiles`, `tags`) without rewriting unrelated fields or the Markdown body.

`references` accepts document IDs and absolute `http(s)` URLs. Document IDs resolve against the workspace, and an unresolved ID is a warning; absolute URLs are opaque loose links that Tandem never fetches, never rewrites, and never warns about. Repository paths are path metadata for `relatedFiles`, which is not validated. Reference warnings cover active Board records (Tasks and Decisions); archived Logs are immutable history and never emit missing-target warnings.

Rules use composite IDs such as `always-12`, a category (`always`, `never`, `prefer`, or `context`), optional `source`, timestamps, and rule text as the Markdown body. Reclassification is delete-and-add.

## Accord and archive

Accord statuses are `ready`, `claimed`, `delivered`, `rework`, `blocked`, and terminal `accepted` or `failed` in Logs. `ready` requires acceptance criteria. `claim` sets top-level `assignee`; `release` clears it and returns to `ready`; `deliver` requires a summary and at least one evidence entry containing non-whitespace text; `resume` changes blocked to claimed. `complete` atomically accepts delivered work and archives it. `fail` atomically archives failed work. `cancel` archives canceled work. Archived records retain the full Task and Accord plus `archivedAt` and minimal `resolution: { outcome, note, reviewer }`; delivery evidence is not duplicated.

Completing a Task warns when its own Accord is neither delivered nor accepted. One narrow exception exists: a grouping Epic (`kind: epic`) whose resolved child hierarchy is nonempty and whose every descendant Task or Subtask is archived with an explicit canonical `resolution.outcome: completed` is eligible to close without that warning. Eligibility only suppresses the missing-delivery warning: for an otherwise-undelivered eligible Epic, completion adds no synthetic Accord delivery or acceptance and copies no child evidence, while normal archive and checkpoint behavior still runs and an explicitly delivered parent still becomes accepted under the ordinary rule. Every other completion check is unchanged: active Board descendants still block closure, unresolved blockers and structural validation still fail, and an empty Epic and any absent, legacy-only, unknown, canceled, or failed descendant outcome retain the ordinary warning.

Archive outcome and delivery status are distinct. `resolution.outcome` records how an archived record ended (`completed`, `canceled`, or `failed`) and is not an Accord status; a missing or legacy-only `completion.outcome` record is not positive evidence of completion and does not qualify for child-based closure.

Validation is exceptional human escalation: `review <id>` requires the exact unresolved `criterion` and a `note`, enters `state: validation`, and may include a reviewer. Completion accepts; Accord rework returns to `in-progress`. Review status is not stored.

`accord release <id> --note <text>` returns work to `ready`, resets workflow `state` to `todo`, and clears its assignee so the Task is claimable again. It accepts an optional `--disposition reassign|discarded`, defaulting to `reassign`; disposition is event data on `accord.released`, not an Accord status. Read projections derive `attemptCount` from `accord.claimed` events, `reworkCount` from `accord.rework` events, and `discardedCount` from released events whose disposition is `discarded`. Counts cover every attempt for the Task, including archived Tasks, and are not editable document fields. Adapters should record post-delivery corrections with `accord rework` rather than out-of-band messages so the counts remain accurate.

## Events

Every durable mutation writes one event to `.tandem/events/<actor-id>.jsonl`; reads never write. The required envelope is:

```json
{"ts":"...","seq":4,"actor":"...","event":"task.updated","id":"task-12","data":{"fields":["tags"]}}
```

Required fields are `ts`, `seq`, `actor`, `event`, `id`, and structured event-specific `data`. Full bodies are never copied into events.

## Native Git checkpoints

The native Rust application writes records and events immediately and never
runs Git during a Task, record, or event mutation. Lifecycle outcomes report a
constant `checkpoint.status` of `batched`: the metadata is durable in the
worktree but pending for the next explicit flush. No mutation stages, commits,
or rewrites anything, and no lifecycle action can produce or absorb a commit.

Metadata is expected repository state. A host workflow collects it at a
deliberate commit/push boundary by running `tandem checkpoint`; this batching
is a host responsibility, not a command the user must remember per mutation.

Plain `tandem checkpoint` is the forward-only native flush. It stages only
the owning `.tandem/` path (`git add -A -- .tandem`), and when that path has
staged changes records them in one ordinary commit with the fixed subject
`chore(tandem): checkpoint metadata`. It never amends, rebases, folds, or
otherwise rewrites an existing commit, so pre-existing source commit
identities are stable across a flush. It creates at most one commit per
invocation and appends it as a new child of the current HEAD. A clean
`.tandem/` is an idempotent no-op: it reports `clean` and creates no commit.
Unrelated staged entries, unstaged bytes, and untracked files are never
touched. A lock in Git's common directory serializes linked worktrees as well
as ordinary processes.

Because a flush is a new commit rather than a history rewrite, a source-only
branch integrates over pending target metadata without stale-base replay, and
genuine overlapping metadata edits still surface as ordinary conflicts. The
explicit command fails closed: a failure exits `1` with a `checkpoint` error
envelope and leaves the pending `.tandem/` change staged for inspection, so
`tandem checkpoint && git push` cannot proceed on failure.

At the push boundary only, a host may explicitly run `tandem checkpoint
--consolidate`. It first performs the ordinary forward-only flush, then
inspects `@{upstream}..HEAD`. Fixed-subject commits changing exclusively the
owning `.tandem/` path are collapsed into one final checkpoint commit after
replaying real commits in order. Its resulting HEAD tree equals the flushed
HEAD tree; eligible unpushed real commits acquire new IDs. The JSON success
checkpoint includes `status: "consolidated"`, `oldHead`, `newHead`, `commit`
(the new HEAD), and `collapsed` (eligible commits); with none eligible it
reports zero and leaves HEAD unchanged. The command refuses with a checkpoint
error envelope and no history rewrite if there is no upstream, the upstream
is not an ancestor, the owning `.tandem/` path is still dirty after the
flush, Git has an operation in progress, the range has a merge or a real
commit touching `.tandem/`, or another local branch or linked worktree is
based inside the range. Unrelated staged entries, unstaged edits, and
untracked files are preserved: replay uses a private index and the new HEAD
has exactly the flushed HEAD tree. Signed commits are also refused rather
than silently losing signatures. Its branch update is atomic and conditioned
on the original HEAD. Never run
this mode on lifecycle writes, ordinary commit boundaries, or published
history; the default checkpoint remains forward-only.

See [`plan/task-40-pi-handoff.md`](../plan/task-40-pi-handoff.md) for the
current lifecycle/flush consumer contract and the required host commit/push
boundary wiring. The earlier
[`plan/task-14-pi-handoff.md`](../plan/task-14-pi-handoff.md) and
[`plan/task-39-pi-handoff.md`](../plan/task-39-pi-handoff.md) contracts are
superseded by it.

## CLI contract

The Rust CLI is clap-derived. The exact command tree is documented by generated help and has 25 leaves: `init`; `add task|decision`; `show`; `assignment`; `list`; `search`; `update`; `accord claim|deliver|rework|block|resume|release|fail`; `review`; `complete`; `cancel`; `checkpoint`; `rules list|add|edit|delete`; `tui`; and `web`. Global `-j/--json`, `-h/--help`, and `-V/--version` work before or after subcommands. JSON success and operational/usage errors are stdout-only envelopes. Human results use stdout and warnings/errors use stderr. Exit codes are 0 success, 1 operational failure, and 2 usage failure. `checkpoint` accepts the explicit `--consolidate` push-boundary flag and reports success as `data.checkpoint` and a checkpoint failure as `{"ok":false,"error":{"code":"checkpoint","message":"...","details":{"checkpoint":{"status":"failed",...}}}}` with exit code 1.

`assignment <task-id> --json` returns the complete current Task and direct milestone definition with an opaque freshness token and derived blocker readiness; assignment nodes also include derived `attemptCount`, `reworkCount`, and `discardedCount`; see [`assignment.md`](assignment.md). A planned validation beginning with `$ ` is a runnable command: after removing the prefix and leading whitespace, the command runs from the Task repository root and exit 0 passes. Other planned validations are manual checks. Assignment JSON classifies each item as `{ "kind": "command" | "manual", "text": "..." }` and strips `$ ` from command text; `show --json` preserves the raw validation strings. Adapters should require captured command output for command entries and may refuse integration on a non-zero exit. Tandem does not execute these commands, and Tasks with no command entries are unaffected. `list` and `search` support `--scope active|archived|all`, defaulting to active. `update` never mutates state, assignee, or Accord status. Repeated list values replace the complete list; absent values remain unchanged; `--clear` removes lists and optional scalars. Human prose accepts leading hyphens while typed IDs, enums, and numbers remain strict.

There is no `upgrade`, `migrate`, `move`, `version` command, `log`, `decision`, or `papercut` command family, compatibility parser, fallback, or adapter implementation in this cutover.
