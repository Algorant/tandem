---
title: CLI Reference
description: Complete reference for the tandem command-line interface.
---

# CLI Reference

The `tandem` binary manages Tandem workspaces from the command line. This page is
an implementation-facing reference for the v0 command surface. It documents
syntax, defaults, filters, state and accord transitions, output modes, and
failure behavior. Commands discover the nearest `.tandem/` workspace from the
current directory; v0 has no workspace-path override.

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

- Workspace: [`init`](#tandem-init), [`upgrade`](#tandem-upgrade)
- Board documents: [`list`](#tandem-list), [`show`](#tandem-show), [`add`](#tandem-add), [`move`](#tandem-move), [`update`](#tandem-update), [`complete`](#tandem-complete), [`cancel`](#tandem-cancel), [`search`](#tandem-search)
- Friction inbox: [`papercut`](#tandem-papercut)
- History: [`log`](#tandem-log)
- Work agreements: [`accord`](#tandem-accord)
- Coordination rules: [`rules`](#tandem-rules)
- Decisions: [`decision`](#tandem-decision)
- Terminal UI: [`tui`](#tandem-tui)
- Local browser UI: [`web`](#tandem-web)
- Version: [`version`](#tandem-version), [`--version`](#tandem---version)

## `tandem` v0 command reference

This section is the implementation-facing CLI reference for v0. Syntax examples use canonical command names and long flags only. V0 commands auto-discover the `.tandem/` workspace from the current directory; an explicit workspace-path override is not part of the locked v0 surface.

### Global CLI conventions

- Human-readable output is the default.
- Compact tables are used for list/search commands.
- Labeled detail blocks are used for show/log/decision detail commands.
- All read commands support `--json` and return this envelope:

```json
{
  "ok": true,
  "data": {},
  "warnings": []
}
```

- JSON read failures should return non-zero and may use the same envelope shape with `ok: false` and an error object in `data`.
- Mutation commands are human-readable in v0; structured mutation output is not required.
- Empty/no-match read behavior:
  - human-readable list/search commands print an explicit empty message and exit `0`.
  - JSON read commands return empty arrays/count objects inside the normal `{ "ok": true, ... }` envelope and exit `0`.
  - missing requested IDs are errors, not no-match results.
- Exit behavior:
  - success exits `0`.
  - usage/argument errors exit `2`.
  - runtime, data, validation, missing-workspace, missing-document, parse, write, and event-append failures exit `1` in the current CLI implementation.
  - warnings do not make a command fail unless paired with a structural error.
- Error wording prefixes recoverable categories where possible: `Parse failure`, `Validation failed`, `Write conflict`, `Write failure`, and `Event append failure`. Event append failures note that the file mutation may already be on disk and needs inspection/repair.

### `tandem upgrade`

- Purpose: explicitly upgrade a discovered legacy Tandem workspace to the current protocol version.
- Kind: mutation.
- Syntax:

```text
tandem upgrade
```

- Accepted options: none. Passing any option or extra argument is a usage error.
- Workspace and upgrade rules:
  - Tandem discovers the nearest `.tandem/` workspace from the current directory.
  - A workspace using protocol `0.1.0` is upgrade-only. Ordinary commands do not upgrade it implicitly; run this command explicitly first.
  - A workspace already using the current protocol `0.2.0` is left unchanged and reports that it is already current.
  - Unsupported protocol versions are rejected. The command does not guess or perform multi-version upgrades.
- Upgrade behavior: updates the workspace protocol version from `0.1.0` to `0.2.0` and canonicalizes legacy `priority: med` and `priority: normal` values to `medium` in active documents and Logs. It preserves document bodies, unknown frontmatter, legacy declarations, and other content. Preserved custom types become read-only after upgrade.
- Success output:

```text
Upgraded Tandem project protocol: 0.1.0 -> 0.2.0
Preserved existing content while canonicalizing legacy `med` and `normal` priorities to `medium` in documents and logs.
```

  An already-current workspace prints `Tandem project is already at protocol 0.2.0.`. Example: `tandem upgrade`.
- Error behavior: fails when no workspace is discoverable, the workspace cannot be read or written, its protocol version is unsupported, or a concurrent file change creates a write conflict. A failed upgrade is not implicit or silently retried by another command.

### `tandem init`

- Purpose: create a new Tandem workspace in the current project.
- Kind: mutation.
- Syntax:

```text
tandem init [--title <title>] [--force]
```

- Required inputs: none.
- Optional inputs:
  - `--title <title>`: explicit workspace title override; when omitted, the title is derived from the current directory basename with `Tandem Workspace` as a fallback.
  - `--force`: overwrite existing Tandem workspace files after user intent is explicit.
- Human output shape: labeled summary of created paths and default states.
- Exit/error notes:
  - fails if a workspace already exists and `--force` is not present.
  - fails on file creation or write errors.

### `tandem list`

- Purpose: list active task and decision documents from the board.
- Kind: read.
- Syntax:

```text
tandem list [--state <state>] [--type <type>] [--priority <priority>] [--tag <tag>] [--assignee <name>] [--parent <id>] [--accord <status>] [--review <status>] [--json]
```

- Required inputs: none.
- Optional inputs:
  - filters: `--state`, `--type`, `--priority`, `--tag`, `--assignee`, `--parent`, `--accord`, `--review`.
  - `--parent <id>` selects documents whose `parentId` matches exactly, whether the parent is a task or another Tandem document type.
  - `--json`: emit structured output.
- Human output shape: compact table grouped or sorted by state. Resolve hierarchy from documents: direct Epic children use `epic-task`, children of Tasks use `subtask`, and valid non-task targets use generic `parent`. Never classify from ID shape.

```text
ID      STATE        TYPE  KIND  RELATION  PARENT      TITLE                ASSIGNEE
task-7  in-progress  task  epic  -         -           Launch docs epic     pi
task-8  validation   task  -     epic-task task-7      Add decision view    pi
task-9  todo         task  -     parent    decision-2  Apply chosen policy  pi
```

- `--json` data shape:

```json
{
  "ok": true,
  "data": {
    "items": [
      {
        "id": "task-8",
        "type": "task",
        "title": "Add decision view",
        "state": "validation",
        "priority": "high",
        "assignee": "pi",
        "parentId": "task-7",
        "parentRelationship": "epic-task",
        "tags": ["tui"],
        "accord": { "status": "delivered" },
        "review": { "status": "pending" }
      }
    ],
    "counts": {
      "total": 1,
      "byState": { "validation": 1 }
    }
  },
  "warnings": []
}
```

- Exit/error notes:
  - fails on missing workspace, invalid filter value, or parse/structure errors.

### `tandem show`

- Purpose: show one active or completed document by ID.
- Kind: read.
- Syntax:

```text
tandem show <id> [--json]
```

- Required inputs:
  - `<id>`: task or decision ID.
- Optional inputs:
  - `--json`: emit structured output.
- Human output shape: labeled detail block with metadata, body, accord/review data, references, and path. A direct child of an Epic uses Task-of-Epic language; a Subtask uses `Subtask of`; other valid parent targets use `Parent`. Showing an Epic exposes direct `Tasks`; showing a Task exposes direct `Subtasks`; Subtasks and non-task documents expose no child collection.
- `--json` data shape includes `document.parentId` plus computed `data.parentRelationship: "epic-task" | "subtask" | "parent"` when the document has a resolved parent. A computed `data.tasks` array is emitted for an Epic, `data.subtasks` for a Task, and neither for a Subtask or non-task document:

```json
{
  "ok": true,
  "data": {
    "document": {
      "id": "task-7",
      "type": "task",
      "kind": "epic",
      "title": "Launch docs epic",
      "state": "in-progress",
      "priority": "high",
      "tags": ["tui"],
      "accord": { "status": "claimed" },
      "review": { "status": "not-ready" }
    },
    "tasks": [
      {
        "id": "task-8",
        "title": "Add decision view",
        "state": "validation",
        "location": "board"
      }
    ],
    "body": "## Description\nCoordinate docs launch.",
    "path": ".tandem/board/task-7.md",
    "location": "board"
  },
  "warnings": []
}
```

- Exit/error notes:
  - fails when the ID is not found in active board documents or completed logs.

### `tandem add`

- Purpose: create a new task in an active state.
- Kind: mutation.
- Syntax:

```text
tandem add --title <title> [--state <state>] [--kind epic] [--description <text>] [--priority <priority>] [--effort <effort>] [--tag <tag>] [--assignee <name>] [--due-date <date>] [--parent <id>] [--blocker <id>] [--reference <ref>] [--related-file <path>] [--json]
```

- Required inputs:
  - `--title <title>`.
- Optional inputs:
  - `--state <state>` defaults to `todo`.
  - `--kind epic`: mark the new root task as an Epic while preserving `type: task` and the task ID namespace. `--kind epic` and `--parent` cannot be combined because Epics cannot have parents.
  - `--parent <id>`: create a normal task linked through canonical `parentId`. A resolved Epic parent creates a global-ID Task with `epic-task`; a resolved Task parent creates a leaf `task-N-M` Subtask with `subtask`; a decision/custom parent creates a global-ID Task with generic `parent`. Attaching beneath a Subtask is an error. Global Epic/Task allocation and per-Task Subtask suffix allocation both scan active board documents and completed logs and reserve without overwriting.
  - metadata: `--description`, `--priority`, `--effort`, repeated `--tag`, `--assignee`, `--due-date`, repeated `--blocker`, repeated `--reference`, repeated `--related-file`. `--effort` records the project's effort value without changing workflow state.
  - `--subtask <title>` is a deprecated inline-checklist authoring path and returns usage guidance to create another task with `--parent` instead. Existing inline `subtasks` metadata remains readable for compatibility.
- Human output shape: labeled created-task summary with ID, state, title, and file path. Epic-parent creation uses Task-of-Epic language, Task-parent creation uses `Created subtask`/`Subtask of`, and non-task parents retain `Created task`/generic `Parent`.
- JSON output shape: `--json` emits the standard success envelope with the created document summary, including `parentId` and computed `parentRelationship` when present, path, and warnings.
- Exit/error notes:
  - fails on invalid state, unsupported kind, invalid referenced parent/blocker, a parented Epic, attachment beneath a Subtask, a role/ID mismatch, or failed write. Direct Epic Tasks never receive hierarchical IDs.

### `tandem move`

- Purpose: move an active task to another active state.
- Kind: mutation.
- Syntax:

```text
tandem move <id> --state <state>
```

- Required inputs:
  - `<id>`: task ID.
  - `--state <state>`: target active state.
- Human output shape: one-line status transition plus any synchronized accord transition and path.
- State/accord synchronization:
  - moving a task from `todo` to `in-progress` claims an existing `accord.status: ready` and prints `Accord: ready -> claimed`.
  - moving to `validation` is preferred for delivered work; existing `state: review` files are tolerated as a legacy alias.
  - ambiguous or destructive accord changes are left to explicit `tandem accord ...` commands.
- Exit/error notes:
  - fails if the task is not active, the ID resolves to a non-task document, the state is unknown, structural validation fails, or the write fails.

### `tandem update`

- Purpose: replace the complete Markdown body or edit workflow-orthogonal metadata on an active task without changing state.
- Kind: mutation.
- Syntax:

```text
tandem update <id> [--title <title>] [--body <markdown>] [--kind epic] [--priority <critical|high|medium|low>] [--effort <effort>] [--assignee <name>] [--due-date <date>] [--parent <id>] [--tag <tag>] [--blocker <id>] [--reference <id>] [--related-file <path>]
```

- Required inputs:
  - `<id>`: active board task ID.
- Optional inputs:
  - exact body replacement: `--body <markdown>` replaces all text after the closing frontmatter delimiter. Empty, whitespace-only, multiline, Unicode, and leading-dash values are valid; omission means no body edit.
  - scalar replacements: `--title`, `--kind`, `--priority`, `--effort`, `--assignee`, `--due-date`. `--effort` replaces the effort metadata value.
  - `--parent <id>`: attach or reparent the task by replacing `parentId` after validating the prospective role graph. An Epic target requires the document to remain a global-ID Task with `epic-task`; a Task target would require a matching `task-N-M` Subtask with `subtask`; a decision/custom target requires a global-ID Task with generic `parent`. Reject a parented Epic, a Subtask target, every role/ID mismatch, and any role-changing or ID-invalidating reparenting. The immutable ID is never renamed.
  - append/deduplicated list metadata: repeated `--tag`, `--blocker`, `--reference`, `--related-file`.
- Unsupported by design:
  - no `--state`; use `tandem move <id> --state <state>` for workflow transitions.
  - no update-time `--description`; that flag remains an add-time convenience that creates a Description section. Use `--body` to replace the exact complete Markdown body. Inline `--subtask` authoring is deprecated in favor of a separate task with `--parent`.
  - no accord/review metadata editing via `update`; use `tandem accord ...` for accord lifecycle changes and review/validation flows for `review:` metadata.
  - no clear/remove flags in v0, including no way to clear an existing `parentId`.
  - completed logs are not updated.
- Validation:
  - kind, when set, must be `epic`; an Epic must have no `parentId`.
  - priority must be one of `critical`, `high`, `medium`, or `low`.
  - parent and blockers must resolve to existing documents. The prospective graph must keep Epics root-only, Subtasks childless, Epics/Tasks global-ID, and Subtasks `task-N-M` beneath the matching Task; references warn when unresolved; related files remain path metadata.
- Human output shape: warnings first, then changed metadata fields with old/new values; a body replacement reports only `body: changed` and never echoes body content. If every requested value already exists byte-for-byte, the command prints a clear no-op and does not update `updatedAt` or append an event.
- Mutation notes: raw-source patches preserve unrelated/unknown frontmatter; metadata-only updates preserve the Markdown body, while `--body` replaces it exactly. Real changes update `updatedAt` and append `task.updated`; event summaries name `body` without copying body content.

### `tandem complete`

- Purpose: complete an active task, archive it to logs, and append an audit event.
- Kind: mutation.
- Syntax:

```text
tandem complete <id> --summary <text> [--file-changed <path>] [--validation <text>] [--reviewer <name>]
```

- Required inputs:
  - `<id>`: task ID.
  - `--summary <text>`: completion summary.
- Optional inputs:
  - repeated `--file-changed <path>`.
  - `--validation <text>`: human-readable validation result summary.
  - `--reviewer <name>`.
- Human output shape: warnings first, then completion summary. The current implementation writes `completedAt` plus nested `completion.summary`, `completion.filesChanged`, `completion.validation`, and `completion.reviewer` metadata; read commands still tolerate earlier flat completion fields.

Example warning output:

```text
Warning: task-7 has review.status=pending.
Warning: task-7 has accord.status=delivered, not accepted.
Completing anyway in v0.

Completed task-7
Moved: .tandem/board/task-7.md -> .tandem/logs/task-7.md
Event: task.completed
```

- Exit/error notes:
  - warns but does not fail for missing accepted review or accepted accord in v0.
  - fails when the ID is missing, the document is not completable, the document is already completed, blockers remain unresolved, structure validation fails, or the move/write fails.

### `tandem cancel`

- Purpose: archive an active Task as canceled while retaining its ID, body, metadata, references, and audit history.
- Kind: mutation.
- Syntax:

```text
tandem cancel <id> --reason <text>
```

- Required inputs:
  - `<id>`: active Task ID.
  - `--reason <text>`: non-empty human explanation.
- Behavior:
  - rejects non-Tasks, archived-only IDs, duplicate Log destinations, invalid hierarchy, and any active descendant;
  - does not cascade and does not require resolved blockers or accepted review/accord;
  - preserves raw body/frontmatter, removes active `state`, updates `updatedAt`, sets compatible archive timestamp `completedAt`, and writes `completion.outcome: canceled` plus `completion.summary: "Canceled: <reason>"`;
  - emits `task.canceled`; a canceled blocker is terminal/resolved, but canceled work is excluded from successful-completion progress;
  - retains the ID in Logs, so existing allocation rules prevent reuse.
- Human output shape: canceled ID, reason, Board-to-Logs path, and event name.
- JSON/Log/TUI reads expose `canceled`; legacy Logs without `completion.outcome` default to `completed`.
- Out of scope: permanent deletion, cascades, same-ID recreation, a dedicated recreate command, and a TUI cancellation action. TUI read/render compatibility is required.

### `tandem log`

#### `tandem log list`

- Purpose: list archived completed and canceled Log documents.
- Kind: read.
- Syntax:

```text
tandem log list [--limit <count>] [--json]
```

- Required inputs: none.
- Optional inputs:
  - `--limit <count>`: maximum rows to show.
  - `--json`: emit structured output.
- Human output shape: compact table sorted by most recent archive timestamp.

```text
ID      ARCHIVED             OUTCOME    TITLE                    SUMMARY
task-7  2026-06-26 15:00     completed  Implement theme loader   Theme loader complete
```

- `--json` data shape:

```json
{
  "ok": true,
  "data": {
    "items": [
      {
        "id": "task-7",
        "type": "task",
        "title": "Implement theme loader",
        "completedAt": "2026-06-26T15:00:00Z",
        "outcome": "completed",
        "summary": "Theme loader complete",
        "accordStatus": "accepted",
        "validationStatus": "passed"
      }
    ],
    "count": 1
  },
  "warnings": []
}
```

#### `tandem log show`

- Purpose: show one completed or canceled Log document.
- Kind: read.
- Syntax:

```text
tandem log show <id> [--json]
```

- Required inputs:
  - `<id>`: archived Task ID.
- Optional inputs:
  - `--json`: emit structured output.
- Human output shape: labeled completion detail block with body, completion metadata, accord evidence, validation, files changed, and timeline where available.
- `--json` data shape:

```json
{
  "ok": true,
  "data": {
    "document": {
      "id": "task-7",
      "type": "task",
      "title": "Implement theme loader",
      "completedAt": "2026-06-26T15:00:00Z"
    },
    "completion": {
      "outcome": "completed",
      "summary": "Theme loader complete",
      "filesChanged": ["src/tui/theme.rs"],
      "validation": { "status": "passed", "summary": "cargo test passed" },
      "reviewer": "Algorant"
    },
    "accord": { "status": "accepted" },
    "body": "## Description\nBuild the theme loader.",
    "events": [
      { "ts": "2026-06-26T15:00:00Z", "event": "task.completed", "id": "task-7", "summary": "Theme loader complete" }
    ]
  },
  "warnings": []
}
```

#### `tandem log search`

- Purpose: search completed and canceled Logs only.
- Kind: read.
- Syntax:

```text
tandem log search <query> [--json]
```

- Required inputs:
  - `<query>`.
- Optional inputs:
  - `--json`: emit structured output.
- Human output shape: compact search table with matching context.
- `--json` data shape:

```json
{
  "ok": true,
  "data": {
    "query": "theme",
    "results": [
      {
        "id": "task-7",
        "title": "Implement theme loader",
        "completedAt": "2026-06-26T15:00:00Z",
        "match": "Summary: Theme loader complete"
      }
    ]
  },
  "warnings": []
}
```

### `tandem search`

- Purpose: search active documents and completed logs.
- Kind: read.
- Syntax:

```text
tandem search <query> [--state <state>] [--type <type>] [--parent <id>] [--json]
```

- Required inputs:
  - `<query>`.
- Optional inputs:
  - `--state <state>` filters active board results.
  - `--type <type>` filters by document type.
  - `--parent <id>` filters active and completed results to documents with that parent, including generic non-task parent targets.
  - `--json`: emit structured output.
- Human output shape: compact table with location (`board` or `logs`), type, optional kind marker, resolved `RELATION` (`epic-task`, `subtask`, or generic `parent`), parent ID, and match snippet.
- `--json` data shape:

```json
{
  "ok": true,
  "data": {
    "query": "theme",
    "results": [
      {
        "id": "task-8",
        "type": "task",
        "title": "Add theme preview",
        "location": "board",
        "state": "in-progress",
        "parentId": "task-7",
        "parentRelationship": "epic-task",
        "snippet": "Add theme preview to the docs launch."
      },
      {
        "id": "task-2",
        "type": "task",
        "title": "Choose theme colors",
        "location": "logs",
        "completedAt": "2026-06-25T18:00:00Z",
        "snippet": "Summary: Theme palette chosen."
      }
    ]
  },
  "warnings": []
}
```

### `tandem papercut`

Papercuts record small, non-blocking friction without creating Tasks or entering Board workflow. Use a Task instead when corrective work needs planning or ownership. Use the blocking lifecycle when work cannot continue.

```sh
tandem papercut add --title "Edit errors hide ambiguous matches" \
  --body "The workaround is to search for each source location first." \
  --tag tooling \
  --reference task-173

tandem papercut list
tandem papercut list --status resolved --json
tandem papercut list --all
tandem papercut show papercut-1 --json
tandem papercut resolve papercut-1 \
  --note "The error now lists all ambiguous source locations." \
  --reference task-201
```

`list` shows open Papercuts by default. Use one `--status open|resolved` filter or `--all`; do not combine them. `show` returns metadata, body, path, and `location: papercuts`. `resolve` updates the same file, requires a note, and can append references. Duplicate titles are valid, and the MVP has no delete or reopen command.

`list` and `show` support the standard JSON envelope. Global `tandem search` finds Papercut title, body, status, tags, references, and resolution note and reports `location: papercuts`. Papercuts never appear in `tandem list`, Logs, hierarchy, Accord, review, completion progress, or the TUI.

### `tandem accord`

- Purpose: manage the work agreement attached to a task.
- The six mutating actions are `claim`, `deliver`, `accept`, `rework`, `block`,
  and `fail`. `ready` is a legacy status that remains readable, but is not an
  action accepted by the current CLI. Use `claim` to start an accord.

- Kind: mutation.

Subcommands:

```text
tandem accord ready <id> [--assignee <name>] [--deliverable <spec>] [--validation <command>] [--constraint <text>]
tandem accord claim <id> --assignee <name>
tandem accord deliver <id> --summary <text> [--evidence <text>] [--file-changed <path>]
tandem accord accept <id> [--reviewer <name>] [--note <text>]
tandem accord rework <id> --note <text>
tandem accord block <id> --reason <text>
tandem accord fail <id> --reason <text>
```

- Required inputs:
  - all subcommands require `<id>`.
  - `claim` requires `--assignee`.
  - `deliver` requires `--summary`.
  - `rework` requires `--note`.
  - `block` and `fail` require `--reason`.
- Optional inputs:
  - `ready` may include repeated `--deliverable`, repeated `--validation`, repeated `--constraint`, and `--assignee`.
  - `deliver` may include repeated `--evidence` and repeated `--file-changed`.
  - `accept` may include `--reviewer` and `--note`.
- Human output shape: labeled status transition plus any synchronized workflow-state transition or state/review warnings. The current implementation writes `accord.claimedAt` on claim, `accord.deliveredAt` on deliver, and repeated `--validation` values under `accord.validation.commands`; it still reads earlier `accord.validations` values.
- State synchronization is conservative: `claim` moves `todo` to `in-progress`; `deliver` and `accept` move compatible `todo`, `in-progress`, or legacy `review` tasks to `validation`; `rework` moves compatible `validation`/legacy `review` tasks back to `in-progress`; `block` and `fail` remain cross-cutting signals and do not automatically move workflow state.

Examples:

```text
tandem accord ready task-7 --assignee pi --deliverable file:src/tui/theme.rs:Theme loader --validation "cargo test"
tandem accord deliver task-7 --summary "Theme loader implemented" --evidence "cargo test passed" --file-changed src/tui/theme.rs
tandem accord rework task-7 --note "Please add no-color fallback."
```

- Exit/error notes:
  - fails if the task is missing, the target is not an active task, existing task/accord/review structure is invalid, the requested accord transition is invalid, required inputs are missing, or the write fails.

### `tandem rules`

#### `tandem rules list`

- Purpose: list project rules.
- Kind: read.
- Syntax:

```text
tandem rules list [--category <category>] [--json]
```

- Required inputs: none.
- Optional inputs:
  - `--category <always|never|prefer|context>`.
  - `--json`: emit structured output.
- Human output shape: grouped rules by category.
- `--json` data shape:

```json
{
  "ok": true,
  "data": {
    "rules": {
      "always": [
        { "id": 1, "rule": "Run tests before completing tasks.", "source": "decision-1" }
      ],
      "never": [],
      "prefer": [],
      "context": []
    },
    "counts": { "always": 1, "never": 0, "prefer": 0, "context": 0, "total": 1 }
  },
  "warnings": []
}
```

#### Rule mutations

- Purpose: add, edit, and delete project rules.
- Kind: mutation.
- Syntax:

```text
tandem rules add --category <category> --rule <text> [--source <id>]
tandem rules edit --category <category> --id <rule-id> --rule <text> [--source <id>]
tandem rules delete --category <category> --id <rule-id>
```

- Human output shape: one-line success plus category and rule ID.
- Examples:

```text
tandem rules add --category always --rule "Run tests before completing tasks." --source decision-1
tandem rules edit --category always --id 1 --rule "Run tests before completing task changes."
tandem rules delete --category always --id 1
```

- Exit/error notes:
  - fails on invalid category, missing rule ID, missing rule text, unresolved required source if treated as structural, or write failure.

### `tandem decision`

#### `tandem decision list`

- Purpose: list decision documents.
- Kind: read.
- Syntax:

```text
tandem decision list [--json]
```

- Required inputs: none.
- Optional inputs:
  - `--json`: emit structured output.
- Human output shape: compact table with ID, ADR status, date, title, references, and first-line summary. `status` is decision metadata, not task workflow `state`.
- `--json` data shape:

```json
{
  "ok": true,
  "data": {
    "items": [
      {
        "id": "decision-1",
        "type": "decision",
        "title": "Use styled-basic Markdown in v0",
        "status": "accepted",
        "date": "2026-06-26",
        "deciders": ["Algorant"],
        "context": "The TUI needs a deterministic v0 Markdown scope.",
        "consequences": ["Advanced Markdown blocks remain deferred."],
        "alternatives": ["Add a full Markdown renderer immediately."],
        "supersedes": ["decision-0"],
        "references": ["task-7"],
        "summary": "Record the v0 rendering scope."
      }
    ],
    "count": 1
  },
  "warnings": []
}
```

#### `tandem decision show`

- Purpose: show one decision document.
- Kind: read.
- Syntax:

```text
tandem decision show <id> [--json]
```

- Required inputs:
  - `<id>`: decision ID.
- Optional inputs:
  - `--json`: emit structured output.
- Human output shape: labeled detail block with metadata, references, body, and path.
- `--json` data shape:

```json
{
  "ok": true,
  "data": {
    "decision": {
      "id": "decision-1",
      "type": "decision",
      "title": "Use styled-basic Markdown in v0",
      "status": "accepted",
      "date": "2026-06-26",
      "deciders": ["Algorant"],
      "context": "The TUI needs a deterministic v0 Markdown scope.",
      "consequences": ["Advanced Markdown blocks remain deferred."],
      "alternatives": ["Add a full Markdown renderer immediately."],
      "supersedes": ["decision-0"],
      "references": ["task-7"]
    },
    "body": "## Decision\nUse styled-basic Markdown rendering for v0.",
    "path": ".tandem/board/decision-1.md"
  },
  "warnings": []
}
```

#### `tandem decision add`

- Purpose: create an ADR-compatible `decision` document.
- Kind: mutation.
- Syntax:

```text
tandem decision add --title <title> [--body <markdown>] [--status <proposed|accepted|rejected|deprecated|superseded>] [--date <date>] [--decider <name>] [--context <text>] [--consequence <text>] [--alternative <text>] [--supersedes <decision-id>] [--superseded-by <decision-id>] [--reference <ref>] [--tag <tag>]
```

- Required inputs:
  - `--title <title>`.
- Optional inputs:
  - `--body <markdown>`: recommended ADR-compatible sections are `Status`, `Context`, `Decision`, `Consequences`, `Supersession`, and `References`.
  - `--status <status>` ADR status; defaults to `proposed` when omitted.
  - `--date <date>` ADR decision date; defaults to the current UTC date when omitted.
  - repeated `--decider <name>`.
  - `--context <text>`.
  - repeated `--consequence <text>`.
  - repeated `--alternative <text>`.
  - repeated `--supersedes <decision-id>`.
  - repeated `--superseded-by <decision-id>`.
  - repeated `--reference <ref>`: related tasks, logs, or decisions; include superseded/superseding decision IDs here when they should be visible to current CLI/TUI search.
  - repeated `--tag <tag>`: use tags such as `adr`, `architecture`, or product area names for filtering.
- Human output shape: warnings first, then labeled created-decision summary with ID, status, date, title, and path.
- Example:

```text
body=$(cat <<'MD'
## Status

Accepted.

## Context

The TUI needs a minimal Markdown renderer for v0.

## Decision

Use styled-basic Markdown rendering first.

## Consequences

This keeps the MVP small while preserving room for richer rendering later.

## Supersession

- Supersedes: none
- Superseded by: none
MD
)
tandem decision add --title "Use styled-basic Markdown in v0" --status accepted --date 2026-06-26 --decider Algorant --context "The TUI needs a deterministic v0 Markdown scope." --consequence "Advanced Markdown blocks remain deferred." --alternative "Add a full Markdown renderer immediately." --reference task-7 --tag adr --body "$body"
```

- Exit/error notes:
  - fails on missing title, invalid ADR status, empty metadata flag values, invalid references that are structural errors, or failed write.
  - unresolved `references`, `supersedes`, or `supersededBy` targets are warnings in v0 related-reference semantics.
  - decision documents do not receive a workflow `state`; ADR `status` remains separate from task state filters and board movement.

### `tandem decision update`

Update selected metadata on an active decision. This mutation has no `--json`
mode and does not change task workflow state.

```text
tandem decision update <decision-id> [--title <title>] [--body <markdown>] [--status <status>]
```

At least one of `--title`, `--body`, or `--status` is required. `--body`
replaces the complete Markdown body; `--status` must be one of
`proposed`, `accepted`, `rejected`, `deprecated`, or `superseded`. Example:
`tandem decision update decision-1 --status accepted`.

### `tandem decision withdraw`

Preserve a decision record while marking it withdrawn with a reason.

```text
tandem decision withdraw <decision-id> --reason <text>
```

`--reason` is required and must not be empty. This is a human-readable mutation
with no `--json` mode. Example:
`tandem decision withdraw decision-1 --reason "Superseded by decision-2"`.

### `tandem tui`

- Purpose: launch the interactive terminal UI.
- Kind: interactive.
- Syntax:

```text
tandem tui
```

- Required inputs: none.
- Optional inputs: none in v0.
- Human output shape: enters the TUI; startup errors are plain terminal errors.
- Current implementation slice:
  - launches a Ratatui/crossterm alternate-screen app from the existing `tandem tui` command.
  - renders top-level Board, Logs, Rules, and Decisions tabs in the target Validation workflow; legacy Review-queue code may exist only as transitional implementation detail while task-25/task-30 remove it.
  - renders the Board view from `.tandem/board` using configured states plus an `unfiled` bucket for active documents without a state; Board states are shown as count tabs and the selected state uses the full Board list area instead of simultaneous narrow columns. The default State Board resolves the strict Epic → Task → Subtask graph from one locked board-plus-logs snapshot, collapses valid descendants beneath roots, and uses `Enter`/mouse as the single row activation path for hierarchy expansion and inline previews. Structural hierarchy failures replace both Board arrangements with a persistent actionable diagnostic panel and disable graph-sensitive TUI mutations until reload succeeds.
  - keeps Board keyboard and mouse navigation local to state subviews/items/detail scrolling, sparse one-line rows, reload, help, and safe quit.
  - supports first Board mutations: `a` starts a quick-add title prompt and creates a basic task in the selected/default configured state; `m` opens an explicit configured-state picker for the selected task. Both flows use raw-source write helpers, reload after success, and surface write/validation errors in the status line.
  - renders selected-task Board details with a dedicated read-only Accord section: semantic status styling, assignee/timestamps, deliverables, validation commands, constraints, summary, evidence, files changed, reviewer/note/reason, and CLI/TUI next-action hints while keeping list rows minimal.
  - renders Review as a real read-only filtered queue of active items needing attention, with local list/detail focus, selectable rows, inspection detail, reason badges/lines, accord/review/state/priority metadata, blockers, and CLI action hints.
  - renders the Logs view as a first-class terminal work-history browser: recency-sorted `.tandem/logs/` list, explicit completed/canceled outcome labels, local list/detail focus, selected-log completion/cancellation summary and timestamp, files/validation/reviewer where present, accord/review metadata, Markdown body, raw path, event context, safe per-log load warnings, and `/` search filtering across ID/title/outcome/summary/body/validation/files.
  - renders Rules as grouped `always`/`never`/`prefer`/`context` lists with keyboard selection, local category navigation, and add/edit/delete prompts that reuse the same raw-source rule mutation behavior as the CLI; Rules view code lives in `src/tui/rules.rs`.
  - renders Decisions as a selectable active decision list with local list/body focus, selected metadata/body/path detail, and a basic title/body add prompt that writes `decision` documents; Decisions view code lives in `src/tui/decisions.rs`.
  - loads built-in `default-dark`/`verdigris` semantic palettes, discovers user themes from `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, lets user config in `$XDG_CONFIG_HOME/tandem/config.toml` or `~/.config/tandem/config.toml` select a named built-in or user theme, lets `.tandem/theme.toml` override that selection per workspace, and applies the active palette to Board, Logs, Rules, and Decisions headers, tabs, borders, selection, status lines, priority badges, accord badges, review badges, and detail/Markdown basics.
  - applies user/workspace theme selection and overrides using the documented simple TOML-style keys; invalid or unknown keys become status-line warnings while the active fallback palette remains in use.
  - enables crossterm mouse capture for basic view tabs, Board state tabs/list rows, detail focus, and wheel interactions; drag/drop remains absent.
  - keeps CLI command behavior unchanged outside the TUI entry point.
- Exit/error notes:
  - fails on missing workspace, parse/structure errors that prevent startup, or non-interactive terminal limitations.
  - v0 does not include a separate TUI executable.

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

### `tandem version`

- Purpose: print the installed Tandem version.
- Kind: read.
- Syntax:

```text
tandem version
```

It prints `tandem <version>` and exits `0`. It does not require a Tandem
workspace. Example: `tandem version`.

### `tandem --version`

`--version` is the global spelling of the same version query. It accepts no
value or additional argument, does not require a workspace, and prints the
same `tandem <version>` line. Example: `tandem --version`.
