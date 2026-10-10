---
title: CLI Reference
description: Complete reference for the tandem command-line interface.
---

# CLI Reference

The `tandem` binary manages Tandem workspaces from the command line. This page
documents the command tree of the current release: syntax, defaults, filters,
the task lifecycle, output modes, and failure behavior. Commands discover the
nearest `.tandem/` workspace from the current directory; there is no
workspace-path override. `tandem --help` and `tandem <command> --help` print the
authoritative flag lists.

For installation and a complete first workflow, see the [Quickstart](/quick-start/).
The install command below is enough to get the binary, while the reference that
follows is not a substitute for the end-to-end onboarding guide.

## Install

Install the latest released binary without `sudo`:

```sh
curl -fsSL https://trytandem.dev/install.sh | sh
tandem --version
```

Or install the tagged source with Cargo:

```sh
cargo install --git https://github.com/Algorant/tandem.git \
  --tag tandem-v0.4.2 --path tandem --locked
```

From a local checkout, use `cargo install --path tandem --locked`.

## Command index

- Workspace: [`init`](#tandem-init), [`sync`](#tandem-sync), [`migrate`](#tandem-migrate)
- Records: [`add`](#tandem-add), [`show`](#tandem-show), [`assignment`](#tandem-assignment), [`list`](#tandem-list), [`search`](#tandem-search), [`update`](#tandem-update), [`link`](#tandem-link)
- Task lifecycle: [lifecycle overview](#task-lifecycle), [`accord`](#tandem-accord), [`review`](#tandem-review), [`complete`](#tandem-complete), [`cancel`](#tandem-cancel)
- Coordination rules: [`rules`](#tandem-rules)
- Interfaces: [`tui`](#tandem-tui), [`web`](#tandem-web)
- Kinds: [papercuts](#papercuts)

The complete tree has these leaves: `init`; `add task|decision`; `show`; `assignment`; `list`; `search`; `update`; `accord claim|deliver|rework|block|resume|release|fail`; `review`; `complete`; `cancel`; `link add|remove`; `sync`, `sync status|resolve`; `migrate`; `rules list|add|edit|delete`; `tui`; and `web`. There are no short flags other than the global `-j`, `-h`, and `-V`.

Removed commands: there is no `move`, `log`, `decision`, `papercut`, `upgrade`, or `version` command, no `accord ready` or `accord accept`, and no Task `update --status`; they fail with a usage error. Use the lifecycle commands below, `list|search --scope archived` for completed history, and `add decision` / `list --type decision` / `update <decision-id> --status` for decisions.

## Global conventions

- `-j`/`--json` is global and may appear before or after the subcommand. `-V`/`--version` prints `tandem <version>`; `-h`/`--help` works without a workspace.
- Human output is the default. Results go to stdout; warnings and errors go to stderr. Mutations print a one-line result and a `Sync:` line.
- With `--json`, every command writes one envelope to stdout and nothing else:

```json
{ "ok": true, "data": {}, "warnings": [] }
```

- A failure with `--json` returns the same stream with `ok: false`:

```json
{ "ok": false, "error": { "code": "io", "message": "document not found: nope", "details": {} } }
```

- Exit codes: `0` success; `1` operational failure (missing workspace or document, validation, parse, write); `2` usage failure (unknown command or flag, missing required input). Warnings never change the exit code.
- A list or search with no matches prints nothing and exits `0`; JSON returns `"data": []`. A missing requested ID is an error.
- Every mutation's JSON `data` includes `id` (the record's final ID) and `sync`: `{ "status": "synced|pending|local-only|not-git", "message": null, "renamed": {}, "conflicts": [], "held": [] }`. See [`tandem sync`](#tandem-sync). Lifecycle mutations also report `event` (accord commands) and `recordWritten`.
- Human prose values (titles, bodies, notes) may start with a hyphen; IDs, enums, and numbers are strict.

## Task lifecycle

Three signals stay separate: workflow `state` (`todo`, `in-progress`, `validation`), `accord.status` (`ready`, `claimed`, `delivered`, `rework`, `blocked`, and the terminal `accepted` or `failed`), and the archive outcome of a completed record. Review status is not stored.

No command sets `state` directly and `update` never edits state, assignee, or Accord status. State changes only through:

| Command | Effect |
| --- | --- |
| `accord claim <id> --assignee <name>` | Accord → `claimed`; state `todo` → `in-progress`; sets the assignee. |
| `accord deliver <id> --summary <text> --evidence <text>` | Accord → `delivered`. Needs `--summary` and at least one non-empty `--evidence`. **State is unchanged**: delivered work stays `in-progress`. |
| `review <id> --criterion <text> --note <text>` | The only route to `validation`. `--criterion` must be one of the Task's acceptance criteria, verbatim. Does not change the Accord. |
| `accord rework <id> --note <text>` | Needs a `delivered` (or already `rework`) Accord. Accord → `rework`; state returns to `in-progress`. |
| `accord release <id> --note <text>` | Accord → `ready`; state → `todo`; clears the assignee. |
| `complete <id>` | Accepts a delivered Accord and archives the Task to Logs in one step. |
| `cancel <id> --note <text>` | Archives the Task as canceled. |

There is no `accord accept`: acceptance happens in `complete`. `validation` is an exceptional human escalation, not a step every Task passes through; an agent normally goes `claim` → `deliver` → `complete` (or `rework`).

Completing a Task whose own Accord is neither `delivered` nor `accepted` is allowed by design. It prints a warning and still completes:

```text
Warning: task-7 has accord.status=ready; complete normally follows a delivered Accord.
Completed task-7 (record written; saved locally (no Git remote to sync with))
```

Expect this warning when closing work that was never delivered, such as a Task finished without an Accord, a research Task, or an Epic (see [`complete`](#tandem-complete) for the Epic exception).

Technical capability is not authority: confirm that the assignment or workspace policy authorizes the actor before `review`, `complete`, `cancel`, or `accord fail`.

### `tandem init`

- Purpose: create a Tandem workspace (`.tandem/` with `tandem.md`, `tasks/`, `decisions/`, `rules/`, `logs/`, and `events/`) in the current directory.
- Syntax: `tandem init [--title <title>]`. `--title` sets the workspace title; otherwise it is derived from the directory name.
- Fails with `Tandem workspace already exists at <path>.` when a workspace is present. There is no `--force`.

### `tandem add`

- Purpose: create a Task or a Decision. The title is a positional argument.
- Syntax:

```text
tandem add task <TITLE> --acceptance <text> [--acceptance <text>...] [--body <markdown>] [--kind <epic|research|papercut>] [--priority <priority>] [--effort <effort>] [--tag <tag>...] [--due-date <date>] [--parent <id>] [--blocker <id>...] [--reference <ref>...] [--related-file <path>...] [--constraint <text>...] [--validation <text>...] [--json]
tandem add decision <TITLE> [--body <markdown>] [--decider <name>...] [--supersedes <decision-id>...] [--reference <ref>...] [--tag <tag>...] [--json]
```

- A new Task is created in `todo` with an Accord at `ready`. There is no `--state`, `--description`, or `--assignee`; the assignee is set by `accord claim`.
- `--acceptance <text>` (repeatable) is required for every Task except `--kind papercut`. `--constraint` and `--validation` record the Accord's constraints and planned validations; a validation beginning with `$ ` is a runnable command (see [`assignment`](#tandem-assignment)).
- `--kind <epic|research|papercut>` sets the Task kind while keeping `type: task`. `--kind epic` and `--parent` cannot be combined. A papercut defaults to `low` priority, may be a root Task or a direct child of an Epic, and is never a Subtask. See [Papercuts](#papercuts).
- `--parent <id>` links through canonical `parentId`. An Epic parent creates a global-ID Task (`epic-task`); a Task parent creates a leaf `task-N-M` Subtask (`subtask`); a decision parent creates a global-ID Task with generic `parent`. Attaching beneath a Subtask is an error. Epic/Task IDs are allocated globally and Subtask suffixes per Task, scanning the Board and Logs and never reusing an ID.
- `--reference <ref>` accepts a document ID or an absolute `http(s)` URL. URLs are opaque and never fetched or warned about; an unresolved document ID warns. Repository paths belong in `--related-file`.
- A Decision is created with `status: proposed`. Change its status, deciders, and other metadata with `tandem update <decision-id>`; Decisions have no workflow `state`.
- Output: `Created task` (or `Created decision`) with `ID:`, `Title:`, and `Sync:` lines. `--json` returns `{ "id": "task-8", "sync": {...} }`.
- Warnings: a `--kind research` or `--kind papercut` Task under any `--parent` is allowed but warns: ``a <kind> Task under <parent> is allowed but not recommended; prefer a root Task plus `tandem link add <id> relates-to <parent>` ``.
- Errors: missing acceptance (`add requires at least one --acceptance <text>; only --kind papercut may omit it`, exit 2), an invalid kind (`Validation failed: invalid kind `<value>`; expected one of: epic, research, papercut`), a parented Epic, attachment beneath a Subtask, a papercut under a Task, or an unresolved parent or blocker.

### `tandem show`

- Purpose: show one active or archived record by ID.
- Syntax: `tandem show <id> [--json]`.
- Human output: labeled lines (`ID`, `Type`, `Title`, `Location` (`board` or `logs`), then `State`, `Accord`, `Assignee`, links, and so on where present).
- `--json` `data` is one flat object: `id`, `type`, `title`, `location`, `state`, `kind`, `role`, `priority`, `effort`, `dueDate`, `assignee`, `tags`, `blockers`, `references`, `relatedFiles`, `parentId`, `parentRelationship` (`epic-task`, `subtask`, `parent`, or `null`), `parent`, `children`, `links`, `incomingLinks`, `accord`, `accordStatus`, `attemptCount`, `reworkCount`, `discardedCount`, `resolution`, `completedAt`, `decision` (for Decisions), `validation` (set by `review`), `body`, `createdAt`, and `updatedAt`.
- Hierarchy is resolved from documents, never from ID shape. An Epic exposes its direct Tasks and a Task its direct Subtasks through `children`.
- Fails when the ID is in neither the Board nor Logs.

### `tandem assignment`

- Purpose: return the complete current definition of a Task for a delegated worker.
- Syntax: `tandem assignment <task-id> [--json]`. The root must be a Task, not a Subtask (`assignment root must be a Task: task-1-1 is subtask`).
- `--json` `data`: `definitionToken` (an opaque freshness token), `dependencyReadiness` (`{ allClear, issues }`), `milestones` (the Task's direct Subtasks), and `root` (`id`, `title`, `type`, `role`, `kind`, `state`, `accordStatus`, `acceptance`, `constraints`, `dependencies`, `ownedScope`, `plannedValidation`, `body`, `ready`, `location`, `resolutionOutcome`, `attemptCount`, `reworkCount`, `discardedCount`).
- Each `plannedValidation` item is `{ "kind": "command" | "manual", "text": "..." }`. A validation beginning with `$ ` is a runnable command; its prefix is stripped. Tandem never runs it.

### `tandem list`

- Purpose: list active records. `--scope archived|all` includes Logs.
- Syntax:

```text
tandem list [--scope <active|archived|all>] [--type <type>] [--state <state>] [--kind <epic|research|papercut>] [--priority <priority>] [--effort <effort>] [--tag <tag>] [--assignee <name>] [--parent <id>] [--accord <status>] [--decision-status <status>] [--resolution <outcome>] [--link <type>] [--linked-to <id>] [--limit <count>] [--json]
```

- `--scope` defaults to `active`. There is no `--review` filter. `--type decision` lists Decisions and `--scope archived` lists completed and canceled Tasks.
- `--kind` takes one value and matches Tasks of exactly that kind; a standard Task is never matched, and tags are never read as kinds. An invalid value fails with `Validation failed: invalid kind `<value>`; expected one of: epic, research, papercut`.
- `--parent <id>` selects documents whose `parentId` matches exactly. `--link <type>` selects records with a stored or derived-inverse link of that type, and `--linked-to <id>` selects records linked to that record.
- Human output: one tab-separated line per record, `<id>	<title>`.
- `--json`: `data` is an array of `{ "id", "title" }`.

```json
{ "ok": true, "data": [{ "id": "task-4", "title": "T4" }], "warnings": [] }
```

### `tandem search`

- Purpose: search active records, and with `--scope` the Logs.
- Syntax: `tandem search <query> [--scope <active|archived|all>] [--type <type>] [--state <state>] [--kind <epic|research|papercut>] [--tag <tag>] [--parent <id>] [--limit <count>] [--json]`.
- Human output: `<id>	<title>	<snippet>` per match. `--json`: `data` is an array of `{ "id", "title", "snippet" }`.

### `tandem update`

- Purpose: edit metadata or replace the Markdown body of an active Task or Decision without changing lifecycle state.
- Syntax:

```text
tandem update <task-id> [--title <title>] [--body <markdown>] [--kind <epic|research|papercut>] [--priority <critical|high|medium|low>] [--effort <effort>] [--due-date <date>] [--parent <id>] [--tag <tag>...] [--blocker <id>...] [--reference <ref>...] [--related-file <path>...] [--acceptance <text>...] [--constraint <text>...] [--validation <text>...] [--clear <field>...] [--json]
tandem update <decision-id> [--title <title>] [--body <markdown>] [--status <proposed|accepted|rejected|deprecated|superseded>] [--decider <name>...] [--supersedes <decision-id>...] [--tag <tag>...] [--reference <ref>...] [--related-file <path>...] [--clear <field>...] [--json]
```

- `<id>` is an active Task or Decision. The command resolves the document's actual `type` and rejects flags that do not apply before writing; type is never inferred from the ID prefix. `--status` exists only for Decisions: on a Task it fails with `update flags not valid for task documents: --status`.
- There is no `--state`, `--assignee`, or `--description`. Workflow changes use the [lifecycle commands](#task-lifecycle). Archived records are not updated.
- `--body` replaces everything after the frontmatter exactly; omission leaves the body alone. A Decision body is removed with `--clear body`, and a Decision `--body` must be non-empty.
- Repeated list flags (`--tag`, `--reference`, `--related-file`, `--blocker`, `--acceptance`, `--constraint`, `--validation`, `--decider`, `--supersedes`) replace the complete list; absent lists are unchanged. `--clear <field>` removes a list or optional scalar (for example `tags`, `blocker`, `due-date`, `parent`, `constraint`, `validation`). A field may be set or cleared, never both in one request. Identity fields and, for Decisions, `status` cannot be cleared.
- `--parent <id>` attaches or reparents after validating the prospective role graph: an Epic target needs a global-ID Task, a Task target needs a matching `task-N-M` Subtask, and a decision target needs a global-ID Task. It rejects a parented Epic, a Subtask target, and every role or ID mismatch. IDs are immutable and never renamed.
- Clearing a parent that would change a Subtask into a Task is refused before any write: `update <subtask> --clear parent` fails with `Validation failed: reparenting <id> would change its canonical role from subtask to task; IDs are immutable`.
- Validation:
  - an Epic must have no `parentId`; a papercut may never be a Subtask (`Validation failed: papercut <id> cannot be a Subtask; a papercut must be a root Task or a direct child of an Epic`);
  - only a papercut may have no acceptance: `--clear acceptance` on any other Task fails with `<id> cannot clear acceptance; an active task requires at least one criterion (only a papercut may have none)`, and re-kinding a papercut without acceptance requires `--acceptance` in the same call;
  - priority must be `critical`, `high`, `medium`, or `low`; Decision `status` must be exactly one of its five values, with no padding;
  - parent and blockers must resolve; unresolved document-ID references warn, while absolute `http(s)` URLs never do;
  - a research or papercut Task that ends up under a parent succeeds but warns, as for `add`.
- Decision timestamps: entering `accepted` or `rejected` writes `decidedAt`; leaving a terminal status keeps it; repeating an unchanged status is a no-op. Every real change writes `updatedAt`.
- Output: `Updated <id>: <fields>` (a body change reports only `body` and never echoes content), or `No changes for <id>` when every value already matches. Unrelated and unknown frontmatter and the body are preserved.
- A record held from sync cannot be updated until it is restored; see [`tandem sync`](#tandem-sync).

### `tandem link`

- Purpose: add or remove typed links between records.
- Syntax: `tandem link add <id> <type> <target>` and `tandem link remove <id> <type> <target>`, where `<type>` is `relates-to`, `duplicates`, `fixed-by`, `fixes`, or `supersedes`.
- Links are stored on `<id>`; the target shows the derived inverse (`fixed-by` ↔ `fixes`) in `show`. `list --link <type> --linked-to <id>` filters on them. `complete <id> --fixed-by <target>` records a `fixed-by` link while completing. `links` and `incomingLinks` appear in `show --json`.
- Output: `Linked <id> <type> <target>` or the matching removal line, followed by sync status.

### `tandem accord`

- Purpose: manage the work agreement attached to a Task. Every active Task has an Accord, created at `ready` with the Task's acceptance criteria.
- Syntax:

```text
tandem accord claim   <id> --assignee <name>
tandem accord deliver <id> --summary <text> --evidence <text> [--evidence <text>...] [--file-changed <path>...]
tandem accord rework  <id> --note <text>
tandem accord block   <id> --note <text>
tandem accord resume  <id>
tandem accord release <id> --note <text> [--disposition <reassign|discarded>]
tandem accord fail    <id> --note <text>
```

- All subcommands accept `--json`. `ready` is a stored status but not a command; a Task starts at `ready`. There is no `accept` (see [`complete`](#tandem-complete)) and the note flag is `--note` for `block`, `release`, `fail`, and `rework` (not `--reason`).
- `claim` sets the Accord to `claimed`, sets the top-level `assignee`, and moves `todo` to `in-progress`; claiming an already accepted Accord is rejected.
- `deliver` requires `--summary` and at least one `--evidence` containing non-whitespace text; without it the command exits 2 with `accord deliver requires at least one non-empty --evidence <text>`. It sets `accord.status: delivered` and records the summary, evidence, and changed files. **It does not change `state`**: delivered work stays `in-progress`.
- `rework` requires status `delivered` (or an existing `rework`); otherwise it fails with `accord rework requires current accord.status=delivered; current status is <status>`. A successful rework sets `rework` and returns the Task to `in-progress`. Record post-delivery corrections with `rework` so `reworkCount` stays accurate.
- `block` records a blocker without changing `state`; `resume` returns a `blocked` Accord to `claimed`.
- `release` returns the Task to the claimable pool: Accord `ready`, state `todo`, assignee cleared. `--disposition` defaults to `reassign`; `discarded` marks the attempt as abandoned. Disposition is event data on `accord.released`, not an Accord status. `attemptCount`, `reworkCount`, and `discardedCount` are derived from events and appear in `show --json` and `assignment --json`.
- `fail` atomically archives the Task to Logs with outcome `failed`.
- Output: `Accord <id>: <status> (record written; <sync message>)`. `--json` `data`: `{ "id", "event" (such as `accord.claimed`), "status", "recordWritten", "sync" }`.
- Errors: a missing or archived Task, a transition invalid from the current status, or missing required input.

### `tandem review`

- Purpose: escalate a Task to human validation. It is the only command that moves a Task to `state: validation`; delivering does not.
- Syntax: `tandem review <id> --criterion <text> --note <text> [--reviewer <name>] [--json]`. `review` has no subcommands.
- `--criterion` must exactly match one of the Task's unresolved acceptance criteria. A non-matching value fails with `Review failed: <id> has no acceptance criterion "<value>"; expected one of: "crit one", "crit two"` and exit `1`.
- Review does not change the Accord (a delivered Accord stays `delivered`). It records `validation.criterion`, `validation.note`, `validation.requestedAt`, and optional `validation.reviewer` on the Task and appends a `review.requested` event; there is no stored review status. Subtasks cannot be reviewed. `complete` accepts the delivered work and `accord rework` returns it to `in-progress`.
- Output: `Validation requested for <id> (record written; <sync message>)`. `--json` `data`: `{ "id", "state": "validation", "recordWritten", "sync" }`.

### `tandem complete`

- Purpose: complete an active Task, accept its delivered Accord, archive it to Logs, and append an audit event.
- Syntax: `tandem complete <id> [--note <text>] [--reviewer <name>] [--fixed-by <id>] [--json]`. There is no `--summary`, `--file-changed`, or `--validation`; delivery evidence is recorded by `accord deliver`.
- `--fixed-by <id>` resolves the Task as fixed by another record and stores a `fixed-by` link. Archived records carry `archivedAt` and a minimal `resolution: { outcome, note, reviewer }`.
- A delivered Accord becomes `accepted` in the same atomic step. Any active Task whose hierarchy, blockers, and structure are valid can be completed.
- Warning, not failure: when the Task's own Accord is neither `delivered` nor `accepted`, the command warns `Warning: <id> has accord.status=<status>; complete normally follows a delivered Accord.` and still completes. This is deliberate; no Accord state is invented.
- Epic exception: a grouping Epic (`kind: epic`) with at least one resolved descendant, where every descendant is archived with an explicit `resolution.outcome: completed`, completes without that warning. Eligibility only suppresses the warning: no delivery or acceptance is synthesized and no child evidence is copied. Empty Epics, active descendants, and absent, legacy, canceled, or failed descendant outcomes keep the warning.
- Output: `Completed <id> (record written; <sync message>)`; `--json` `data`: `{ "id", "recordWritten", "sync" }`.
- Fails when the ID is missing or already archived, active descendants remain (`cannot complete <id> while it has active descendants: <ids>`), blockers are unresolved, or structure validation fails.

### `tandem cancel`

- Purpose: archive an active Task as canceled while retaining its ID, body, metadata, references, and audit history.
- Syntax: `tandem cancel <id> --note <text> [--json]`. `--note` is required (there is no `--reason`).
- Rejects non-Tasks, archived IDs, invalid hierarchy, and any active descendant. It does not cascade and does not require resolved blockers or an accepted Accord.
- The record keeps its raw body and frontmatter, drops active `state`, and gains the canceled resolution and `archivedAt`. A canceled blocker is resolved, but canceled work is excluded from successful-completion progress. The ID stays in Logs and is never reused.
- Output: `Canceled <id> (record written; <sync message>)`.
- Out of scope: permanent deletion, cascades, same-ID recreation, and a TUI cancellation action.

### `tandem sync`

- Purpose: synchronize the board with the repository's `tandem` branch now. Tandem already syncs after every change, before reads older than 60 seconds, and in the background of the TUI and web view, so this is rarely needed.
- Kind: mutation of the board and `tandem` branch only; never the source branch, index, or working tree.
- Syntax:

```text
tandem sync
tandem sync status
tandem sync resolve <id> --keep <local|remote|edited>
```

- `tandem sync` fetches, merges record by record, numbers records that still have temporary `<prefix>-new-<hex>` IDs, and pushes without force. Offline, local changes stay saved and the result is `pending`. Without a remote the result is `local-only`.
- `tandem sync status` needs no network. It reports the remote, whether local changes are waiting, the last fetch, the last problem, open conflicts, and held edits. Its holds are computed locally only: a hold that exists because of a change on the remote you have not yet fetched appears only in `tandem sync`.
- `tandem sync resolve` settles a conflict: `local` keeps this machine's version, `remote` keeps the shared version, and `edited` keeps the file as you edited it in `.tandem/`. Changes to a conflicted record are refused until it is resolved.

**Held edits.** A local change that cannot be published is held back and never overwrites the shared board. Held changes include unparsable files, a missing or changed `uid`, an `id` that does not match the file name, a changed `protocolVersion` or `workspaceId`, and a record that parses but would make the shared board invalid (for example an unresolved `parentId`). For a validation hold on a Git board with a remote:

- Reads use the shared version of the record with a warning naming the file (`Warning: reading the shared version of task-1: its local edit .tandem/tasks/task-1.md is held from sync: would make the shared board invalid: <message>`). A held record that was never shared is skipped (`Warning: skipped task-9: its new local record ... is held from sync: ...`) and `show` reports it as not found.
- Mutations of other records still work and report the hold: `Sync: saved locally; pending sync (some changes are held; see `tandem sync status`)` plus a `held:` line.
- Mutating the held record is refused: `<id> has a local edit held from sync (<reason>). Restore the shared version with `tandem sync resolve <id> --keep remote` before changing it.`
- `tandem sync status` prints `Held edit: <path>: would make the shared board invalid: <message>` followed by `  resolve: tandem sync resolve <id> --keep remote`.
- `tandem sync resolve <id> --keep remote` repairs a validation hold when no conflict exists for that ID: it restores the shared version, or removes a record that was never shared. `--keep local` and `--keep edited` fail clearly, because an unpublishable version cannot be kept.

JSON: `tandem sync status --json` returns `data` with `remote`, `git`, `pending`, `published`, `lastFetch`, `lastError`, `conflicts`, and `held`; each `held` entry is `{ "path", "reason", "id" }`. Mutations and `tandem sync --json` return `data.sync` with `status` (`synced`, `pending`, `local-only`, or `not-git`), `message`, `renamed`, `conflicts`, and `held`.

### `tandem migrate`

- Purpose: the single version-stepping command. It reads the board's `protocolVersion` and runs the matching one-time step: a protocol 0.3.0 board is moved to the repository's `tandem` branch, and a protocol 0.4.0 board is upgraded to 0.5.0. Any other version, including a board already at 0.5.0, fails.
- Kind: mutation.
- Syntax:

```text
tandem migrate [--dry-run]
tandem migrate --adopt [--dry-run]
```

- **Install the new Tandem on every machine that shares the board before migrating it to 0.5.0 and syncing.** Older Tandem versions reject the new protocol version and the `research` and `papercut` kinds.
- **0.4.0 → 0.5.0.** Sets `kind` from the `research` or `papercut` tag on active Board Tasks and removes that tag, bumps `protocolVersion`, and publishes both as one commit on the `tandem` branch (the board must be synced and free of conflicts first). Tasks tagged both, tasks that already have a different kind (for example Epics), and papercut-tagged Subtasks are reported and left unchanged; nothing is guessed. Archived Logs, Decisions, and Rules are never rewritten. When another machine already upgraded the shared board, `migrate` downloads it instead (and refuses if this checkout holds unsynced changes). A board without a remote is rewritten locally. Run it once per repository; run it again on each other machine to receive the upgrade.
- **0.3.0 → 0.5.0.** Requires a Git remote without a `tandem` branch, a branch that is not behind its upstream, and no staged changes. It adds a permanent `uid` to every record, applies the same tag-to-kind conversion, publishes the board, and creates one source commit (`chore(tandem): move the Tandem board to the tandem branch`) that stops tracking `.tandem/` and ignores it. Push that commit normally.
- `tandem migrate --adopt` is for another machine that still has unpushed or uncommitted 0.3.0 board changes and a shared board already at 0.5.0. Run it before `git pull`. It merges those changes into the shared board, renumbers records that were never published (reporting old and new IDs), and restores the tracked legacy files so `git pull` can remove them. A machine without local board changes just pulls.
- `--dry-run` reports what would change and changes nothing.
- JSON (`--json`): `{"ok":true,"data":{"dryRun":false,"fromVersion":"0.4.0","toVersion":"0.5.0","remote":"origin","records":12,"files":15,"migratedFrom":null,"sourceCommit":null,"upgraded":"published|received|notSynced|null","kinds":{"converted":[{"id":"task-7","kind":"papercut"}],"skipped":[{"id":"task-9","reason":"tagged both research and papercut"}]}},"warnings":[]}`. `upgraded` is `null` for the 0.3.0 step and on `--dry-run`.
- Errors: `this board is already at protocol 0.5.0; nothing to migrate`; `tandem migrate converts protocol 0.3.0 and 0.4.0 boards; found \`<version>\``. Ordinary commands on a 0.4.0 board fail with `This board uses protocol 0.4.0; this Tandem version requires 0.5.0. Run \`tandem migrate\` to upgrade it. Every machine that shares this board must install this Tandem version before the board is migrated and synced, because older versions cannot read the new kinds or protocol version.`
- See [Upgrading to independent sync](/guides/upgrading-to-independent-sync/) for the 0.3.0 step.

### Papercuts

A papercut is a Task with `kind: papercut`: small, non-blocking friction that caused confusion, avoidable retries, unnecessary effort, or a workaround worth preserving. There is no `papercut` command; use `add task`, `update`, `list --kind papercut`, and `search --kind papercut`.

```sh
tandem add task "Edit errors hide ambiguous matches" --kind papercut --tag tooling
tandem add task "Setup docs omit the env var" --kind papercut --parent task-12 --priority medium
tandem list --kind papercut
tandem search "ambiguous" --kind papercut
```

- A papercut needs only a title. `--acceptance` is optional for it and required for every other Task.
- `priority` defaults to `low`; pass `--priority` to override.
- Placement: a root Task or a direct child of an Epic. A papercut is never a Subtask; creating, reparenting, or re-kinding one into a Subtask fails validation. Nesting a papercut (or research Task) under a parent is allowed but warns; prefer a root Task plus `tandem link add <id> relates-to <parent>`.
- Tags are topical only. `research` and `papercut` tags are not read as kinds; `tandem migrate` converts old tags on active Board Tasks once (see [`tandem migrate`](#tandem-migrate)).
- `show --json` and `assignment <task-id> --json` report `kind` (`data.root.kind` for an assignment; `null` for a standard Task).
- Use a blocking lifecycle when work cannot continue, and a normal Task when the fix needs planning.

### `tandem rules`

- Purpose: list and mutate the project rules stored in `.tandem/rules/`. A rule has a category (`always`, `never`, `prefer`, `context`), an ID such as `always-3`, optional `source`, and text.
- Syntax:

```text
tandem rules list [<category>] [--json]
tandem rules add <category> <text> [--source <id>] [--json]
tandem rules edit <id> <text> [--source <id>] [--clear <field>] [--json]
tandem rules delete <id> [--json]
```

- `category` and `text` are positional, and `edit`/`delete` take the composite rule ID. Reclassifying a rule is delete-and-add. `--clear source` removes the source.
- Human output: `list` prints `<id>	<text>` per rule; mutations print `Created rule <id>`, `Updated rule <id>`, or `Deleted rule <id>`. `--json` `list` returns `data` as an array of rules.
- Examples:

```text
tandem rules add always "Run tests before completing tasks." --source decision-1
tandem rules edit always-1 "Run tests before completing task changes."
tandem rules delete always-1
```

### `tandem tui`

- Purpose: launch the interactive terminal UI.
- Syntax: `tandem tui`. It takes no options and needs an interactive terminal and a workspace.
- Top-level views are Board, Logs, Rules, and Decisions (`1`–`4`). Delivered and validation work is handled through Board states and the Validation actions, not a separate view. See the [TUI guide](/tui/) for navigation, actions, themes, and mouse support.

### `tandem web`

- Purpose: open a local read-only browser view of the nearest workspace.
- Kind: long-running read interface.
- Syntax:

```text
tandem web [--port <port>] [--no-open]
```

Without options, Tandem selects an available loopback port, prints the URL and
project path, and opens the default browser. `--port <port>` selects a specific
port from 1 through 65535. `--no-open` prints and serves the URL without opening
a browser. Press `Ctrl-C` to stop the server.

The server binds only to `127.0.0.1`, serves one discovered workspace, embeds
all browser assets in the binary, and exposes no mutations or remote-bind
option. See the [Web guide](/web/) for available views, refresh behavior,
security boundaries, appearance, accessibility, and deferred capabilities.
