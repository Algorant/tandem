# Tandem CLI/TUI Spec Draft

Status: draft  
Date: 2026-06-26  
Working name: Tandem  
Implementation target: CLI v0 surface complete for current known scope; forward focus Rust + Ratatui/crossterm TUI inside `tandem-tui/`

Naming:

- Product/protocol: **Tandem**
- CLI/TUI source directory: `tandem-tui/`
- CLI binary: `tdm`
- CLI design and current known CLI v0 implementation precede TUI implementation; future CLI work should be explicit new features or bug fixes
- V0 TUI invocation: `tdm tui` only
- User-facing CLI: `tdm`; reserve `td` for future/internal tool prefixes

This document describes the user-facing `tdm` CLI and terminal UI for the Tandem protocol described in `../../protocol/plan/spec.md`.

The CLI/TUI baseline is broad feature parity with the live Brainfile project: keep the general command/workflow shape, then intentionally improve the flawed parts. The intent is not to port the current Brainfile Ink TUI directly. The CLI v0 surface is now implemented for the current known scope; the active implementation focus is a more capable, responsive, themeable, mouse-aware TUI.

## Baseline inputs

- Live Brainfile CLI behavior from `brainfile --help` and subcommand help.
- Installed Brainfile implementation under `/usr/lib/node_modules/@brainfile/cli` for feature discovery.
- Current Brainfile TUI behavior and shortcomings.
- Tandem protocol draft in `../../protocol/plan/spec.md`.
- Local Brainfile v3 direction in `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md`.

Feature parity remains a planning reference. The locked v0 CLI/TUI scope below is binding for the first implementation pass.

## Current Brainfile TUI issues to avoid

Observed/known pain points:

- Progress is tied to a literal `done` column, which does not fit v2 archive/log completion.
- Theming is effectively hardcoded.
- Mouse support is missing.
- Logs exist but are not rich enough to feel like a first-class completed-work view.
- Review/validation/accord status is not prominent enough.
- Many actions are keyboard-only and modal in ways that feel bolted on.
- UI is constrained by the current Ink/React implementation rather than designed around the protocol.

## Product goal

A fast CLI and terminal workspace for managing project work, agent accords, reviews, decisions, rules, and logs.

It should feel closer to a local-first Linear/kanban/logbook hybrid than a simple task list.

## Design principles

- **CLI first, TUI now:** the `tdm` command workflows are implemented for the current known v0 scope; focus new work on the interactive TUI unless fixing CLI bugs or adding requested CLI features.
- **Feature parity baseline:** map live Brainfile features before deciding what Tandem keeps, renames, improves, or omits.
- **Logs are real:** completed work is browsable, searchable, inspectable, and useful; restore/reopen behavior can come after the v0 log read scope.
- **Review is central:** delivered work should naturally flow to review, validation, acceptance, rework, and completion.
- **Agent state is visible:** accord status, evidence, validation, and blockers should be visible without opening raw files.
- **Fast scanning:** compact cards, good color hierarchy, clear badges, and useful filtering.
- **Keyboard-first, not keyboard-only:** vim-style and arrow navigation plus real mouse interactions.
- **Themeable from day one:** no hardcoded palette-only implementation.
- **Small-screen aware:** usable in narrow terminals and inside split panes.
- **File-native:** edits should reflect on disk; external edits should hot reload cleanly.

## Locked v0 CLI scope

The v0 CLI command families are settled:

```text
tdm init
tdm list
tdm show <id>
tdm add ...
tdm move <id> --state <state>
tdm complete <id>
tdm log list|show|search
tdm search <query>
tdm accord ready|claim|deliver|accept|rework|block|fail
tdm rules list|add|edit|delete
tdm decision list|show|add
tdm tui
```

Command behavior rules:

- Design and document the CLI before the interactive TUI.
- Use Tandem vocabulary: `state`, `accord`, and `decision`.
- Human-readable output is the default: list/search commands use compact tables; detail commands use labeled blocks with Markdown body where applicable.
- V0 uses canonical command names and long flags only; abbreviated flags or alias commands are not part of v0.
- All read commands support `--json`: `list`, `show`, `search`, `log list`, `log show`, `log search`, `rules list`, `decision list`, and `decision show`.
- `--json` responses use an envelope object: `{ "ok": true, "data": ..., "warnings": [] }`.
- `tdm log` is limited to `list`, `show`, and `search` in v0.
- `tdm rules` supports `list`, `add`, `edit`, and `delete` in v0.
- `tdm accord` supports `ready`, `claim`, `deliver`, `accept`, `rework`, `block`, and `fail` in v0.
- `tdm decision` supports `list`, `show`, and `add` in v0.
- The TUI launches through `tdm tui` only in v0; no standalone TUI binary is part of v0.
- `tdm complete` moves completed work to logs and warns about missing review or accord acceptance instead of blocking completion in v0.
- The first implementation language is Rust, implemented inside `tandem-tui/`.
- The current implementation package is a Rust binary crate in `tandem-tui/` with manual argument parsing, the approved `yaml-rust2` dependency for frontmatter reads, raw-source CLI mutation patches, and Ratatui/crossterm for the first TUI shell. Completion writes nested `completion` metadata and accord actions write canonical validation/timestamp metadata while preserving legacy read aliases. Additional dependency changes still require an explicit decision.

Deferred from v0:

- Template features.
- Schema-management command surface.
- AI-assistant integration commands.
- Credential/provider commands.
- Third-party archive/export integrations.
- Brainfile conversion commands are not required for v0.
- Schemas, fixtures, and root Rust workspace layout are not part of v0.

## `tdm` v0 command reference

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

### `tdm init`

- Purpose: create a new Tandem workspace in the current project.
- Kind: mutation.
- Syntax:

```text
tdm init --title <title> [--force]
```

- Required inputs:
  - `--title <title>`: workspace title.
- Optional inputs:
  - `--force`: overwrite existing Tandem workspace files after user intent is explicit.
- Human output shape: labeled summary of created paths and default states.
- Exit/error notes:
  - fails if a workspace already exists and `--force` is not present.
  - fails on file creation or write errors.

### `tdm list`

- Purpose: list active task and decision documents from the board.
- Kind: read.
- Syntax:

```text
tdm list [--state <state>] [--type <type>] [--priority <priority>] [--tag <tag>] [--assignee <name>] [--accord <status>] [--review <status>] [--json]
```

- Required inputs: none.
- Optional inputs:
  - filters: `--state`, `--type`, `--priority`, `--tag`, `--assignee`, `--accord`, `--review`.
  - `--json`: emit structured output.
- Human output shape: compact table grouped or sorted by state.

```text
ID      STATE        PRI   TITLE                         ASSIGNEE  ACCORD      REVIEW
task-7  in-progress  high  Implement theme loader        pi        claimed     not-ready
task-8  review       med   Add decision view             pi        delivered   pending
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
        "state": "in-progress",
        "priority": "high",
        "assignee": "pi",
        "tags": ["tui"],
        "accord": { "status": "claimed" },
        "review": { "status": "not-ready" }
      }
    ],
    "counts": {
      "total": 1,
      "byState": { "in-progress": 1 }
    }
  },
  "warnings": []
}
```

- Exit/error notes:
  - fails on missing workspace, invalid filter value, or parse/structure errors.

### `tdm show`

- Purpose: show one active or completed document by ID.
- Kind: read.
- Syntax:

```text
tdm show <id> [--json]
```

- Required inputs:
  - `<id>`: task or decision ID.
- Optional inputs:
  - `--json`: emit structured output.
- Human output shape: labeled detail block with metadata, body, accord/review data, references, and path.
- `--json` data shape:

```json
{
  "ok": true,
  "data": {
    "document": {
      "id": "task-7",
      "type": "task",
      "title": "Implement theme loader",
      "state": "in-progress",
      "priority": "high",
      "tags": ["tui"],
      "accord": { "status": "claimed" },
      "review": { "status": "not-ready" }
    },
    "body": "## Description\nBuild the theme loader.",
    "path": ".tandem/board/task-7.md",
    "location": "board"
  },
  "warnings": []
}
```

- Exit/error notes:
  - fails when the ID is not found in active board documents or completed logs.

### `tdm add`

- Purpose: create a new task in an active state.
- Kind: mutation.
- Syntax:

```text
tdm add --title <title> [--state <state>] [--description <text>] [--priority <priority>] [--tag <tag>] [--assignee <name>] [--due-date <date>] [--parent <id>] [--blocker <id>] [--reference <ref>] [--related-file <path>] [--subtask <title>]
```

- Required inputs:
  - `--title <title>`.
- Optional inputs:
  - `--state <state>` defaults to `todo`.
  - metadata: `--description`, `--priority`, repeated `--tag`, `--assignee`, `--due-date`, `--parent`, repeated `--blocker`, repeated `--reference`, repeated `--related-file`, repeated `--subtask`.
- Human output shape: labeled created-task summary with ID, state, title, and file path.
- Exit/error notes:
  - fails on invalid state, invalid referenced parent/blocker, structure errors, or failed write.

### `tdm move`

- Purpose: move an active task to another active state.
- Kind: mutation.
- Syntax:

```text
tdm move <id> --state <state>
```

- Required inputs:
  - `<id>`: task ID.
  - `--state <state>`: target active state.
- Human output shape: one-line status transition plus path.
- Exit/error notes:
  - fails if the task is not active, the ID resolves to a non-task document, the state is unknown, structural validation fails, or the write fails.

### `tdm complete`

- Purpose: complete an active task, archive it to logs, and append an audit event.
- Kind: mutation.
- Syntax:

```text
tdm complete <id> --summary <text> [--file-changed <path>] [--validation <text>] [--reviewer <name>]
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

### `tdm log`

#### `tdm log list`

- Purpose: list completed log documents.
- Kind: read.
- Syntax:

```text
tdm log list [--limit <count>] [--json]
```

- Required inputs: none.
- Optional inputs:
  - `--limit <count>`: maximum rows to show.
  - `--json`: emit structured output.
- Human output shape: compact table sorted by most recent completion.

```text
ID      COMPLETED            TITLE                    ACCORD    SUMMARY
task-7  2026-06-26 15:00     Implement theme loader   accepted  Theme loader complete
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

#### `tdm log show`

- Purpose: show one completed log document.
- Kind: read.
- Syntax:

```text
tdm log show <id> [--json]
```

- Required inputs:
  - `<id>`: completed task ID.
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
      "summary": "Theme loader complete",
      "filesChanged": ["src/tui/theme.rs"],
      "validation": { "status": "passed", "summary": "cargo test passed" },
      "reviewer": "ivan"
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

#### `tdm log search`

- Purpose: search completed logs only.
- Kind: read.
- Syntax:

```text
tdm log search <query> [--json]
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

### `tdm search`

- Purpose: search active documents and completed logs.
- Kind: read.
- Syntax:

```text
tdm search <query> [--state <state>] [--type <type>] [--json]
```

- Required inputs:
  - `<query>`.
- Optional inputs:
  - `--state <state>` filters active board results.
  - `--type <type>` filters by document type.
  - `--json`: emit structured output.
- Human output shape: compact table with location (`board` or `logs`) and match snippet.
- `--json` data shape:

```json
{
  "ok": true,
  "data": {
    "query": "theme",
    "results": [
      {
        "id": "task-7",
        "type": "task",
        "title": "Implement theme loader",
        "location": "board",
        "state": "in-progress",
        "snippet": "Build the theme loader."
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

### `tdm accord`

- Purpose: manage the work agreement attached to a task.
- Kind: mutation.

Subcommands:

```text
tdm accord ready <id> [--assignee <name>] [--deliverable <spec>] [--validation <command>] [--constraint <text>]
tdm accord claim <id> --assignee <name>
tdm accord deliver <id> --summary <text> [--evidence <text>] [--file-changed <path>]
tdm accord accept <id> [--reviewer <name>] [--note <text>]
tdm accord rework <id> --note <text>
tdm accord block <id> --reason <text>
tdm accord fail <id> --reason <text>
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
- Human output shape: labeled status transition and any state/review warnings. The current implementation writes `accord.claimedAt` on claim, `accord.deliveredAt` on deliver, and repeated `--validation` values under `accord.validation.commands`; it still reads earlier `accord.validations` values.

Examples:

```text
tdm accord ready task-7 --assignee pi --deliverable file:src/tui/theme.rs:Theme loader --validation "cargo test"
tdm accord deliver task-7 --summary "Theme loader implemented" --evidence "cargo test passed" --file-changed src/tui/theme.rs
tdm accord rework task-7 --note "Please add no-color fallback."
```

- Exit/error notes:
  - fails if the task is missing, the target is not an active task, existing task/accord/review structure is invalid, the requested accord transition is invalid, required inputs are missing, or the write fails.

### `tdm rules`

#### `tdm rules list`

- Purpose: list project rules.
- Kind: read.
- Syntax:

```text
tdm rules list [--category <category>] [--json]
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
tdm rules add --category <category> --rule <text> [--source <id>]
tdm rules edit --category <category> --id <rule-id> --rule <text> [--source <id>]
tdm rules delete --category <category> --id <rule-id>
```

- Human output shape: one-line success plus category and rule ID.
- Examples:

```text
tdm rules add --category always --rule "Run tests before completing tasks." --source decision-1
tdm rules edit --category always --id 1 --rule "Run tests before completing task changes."
tdm rules delete --category always --id 1
```

- Exit/error notes:
  - fails on invalid category, missing rule ID, missing rule text, unresolved required source if treated as structural, or write failure.

### `tdm decision`

#### `tdm decision list`

- Purpose: list decision documents.
- Kind: read.
- Syntax:

```text
tdm decision list [--json]
```

- Required inputs: none.
- Optional inputs:
  - `--json`: emit structured output.
- Human output shape: compact table with ID, title, references, and first-line summary.
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
        "references": ["task-7"],
        "summary": "Record the v0 rendering scope."
      }
    ],
    "count": 1
  },
  "warnings": []
}
```

#### `tdm decision show`

- Purpose: show one decision document.
- Kind: read.
- Syntax:

```text
tdm decision show <id> [--json]
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
      "references": ["task-7"]
    },
    "body": "## Decision\nUse styled-basic Markdown rendering for v0.",
    "path": ".tandem/board/decision-1.md"
  },
  "warnings": []
}
```

#### `tdm decision add`

- Purpose: create a decision document.
- Kind: mutation.
- Syntax:

```text
tdm decision add --title <title> [--body <markdown>] [--reference <ref>] [--tag <tag>]
```

- Required inputs:
  - `--title <title>`.
- Optional inputs:
  - `--body <markdown>`.
  - repeated `--reference <ref>`.
  - repeated `--tag <tag>`.
- Human output shape: labeled created-decision summary with ID and path.
- Example:

```text
tdm decision add --title "Use styled-basic Markdown in v0" --body "## Decision\nUse styled-basic rendering first." --reference task-7
```

- Exit/error notes:
  - fails on missing title, invalid references that are structural errors, or failed write.

### `tdm tui`

- Purpose: launch the interactive terminal UI.
- Kind: interactive.
- Syntax:

```text
tdm tui
```

- Required inputs: none.
- Optional inputs: none in v0.
- Human output shape: enters the TUI; startup errors are plain terminal errors.
- Current implementation slice:
  - launches a Ratatui/crossterm alternate-screen app from the existing `tdm tui` command.
  - renders top-level Board, Review, Logs, Rules, and Decisions tabs; `1`..`5` and mouse tab clicks switch views.
  - renders the Board view from `.tandem/board` using configured states plus an `unfiled` bucket for active documents without a state.
  - keeps Board keyboard navigation across states/items, selected-item detail scrolling, reload, help, and safe quit.
  - supports first Board mutations: `a` starts a quick-add title prompt and creates a basic task in the selected/default configured state; `H`/`L` moves the selected task to the previous/next configured state. Both flows use raw-source write helpers, reload after success, and surface write/validation errors in the status line.
  - shows Review, Logs, Rules, and Decisions as read-only placeholders with counts/load warnings so later slices can add full workflows on stable view state.
  - loads the built-in `default-dark` semantic palette, applies it to Board headers, tabs, borders, selection, status lines, priority badges, accord badges, review badges, and detail/Markdown basics.
  - applies a workspace theme override from `.tandem/theme.toml` using the documented simple TOML-style color keys; invalid or unknown keys become status-line warnings while the default palette remains active.
  - enables crossterm mouse capture for basic tab, column/detail, and wheel interactions; drag/drop remains absent.
  - keeps CLI command behavior unchanged outside the TUI entry point.
- Exit/error notes:
  - fails on missing workspace, parse/structure errors that prevent startup, or non-interactive terminal limitations.
  - v0 does not include a separate TUI executable.

## Brainfile parity reference

| Brainfile shape | Tandem v0 decision |
| --- | --- |
| Board/task CRUD | Keep, using `state` and Tandem document IDs. |
| Completed work area | Improve through first-class logs. |
| Contract workflow | Rename and improve as `accord`. |
| Rules categories | Keep: `always`, `never`, `prefer`, `context`. |
| ADR command | Rename around `decision`; v0 subcommands are `list`, `show`, and `add`. |
| Type management | Defer CLI management; protocol config may still define types. |
| Template features | Defer. |
| External service/archive integrations | Defer. |
| Assistant/server integration commands | Defer. |

## First TUI MVP

The first TUI MVP is not read-only. The current starter slices establish the Ratatui/crossterm event loop, render top-level Board/Review/Logs/Rules/Decisions view state, support Board navigation/details/reload/quit, and include small Board mutations: quick-add a basic task with `a`, and move the selected task left/right between configured states with `H`/`L`. Review/Logs/Rules/Decisions currently start as read-only placeholders/counts; full workflows remain for subsequent slices.

The full first TUI MVP should include:

- Top-level views: Board, Review, Logs, Rules, Decisions.
- Board mutations: add item, move state, edit item, complete to logs, update priority/tags/assignee where supported, and toggle subtasks.
- Accord actions: ready, claim, deliver, accept, rework, block, fail.
- Rules actions: list, add, edit, delete.
- Decision browsing and basic decision actions matching `tdm decision list|show|add`.
- First-class logs with list/show/search behavior.
- Theme support in the first MVP, not a later polish pass.
- Mouse support is enabled by default in the first MVP for selection, scrolling, tabs, and action buttons; drag/drop is not in v0.
- Progress/health metrics that do not require a persistent `done` state.

## TUI views

### 1. Board view

Primary view for active Tandem items.

Default states:

```text
todo | in-progress | review
```

Projects may configure state names. The TUI should not assume a persistent completion state exists.

Board view should support:

- state/column tabs or columns depending on layout width
- task/decision cards
- compact and expanded card modes
- priority, type, tags, parent, blocker, assignee, due date badges
- accord status badges
- review status badges
- selection and multi-select later
- click actions when mouse mode is enabled

Card example:

```text
▌ HIGH [task] Implement Ratatui theme system       [A:claimed] [2/5]
    #tui #rust · @pi · child of task-4             task-7
```

Delivered/review item example:

```text
▌ MED  Add decision view                           [A:delivered] [review]
    validation pending · 3 files changed           task-8
```

### Shared detail pane

A focused pane or full-screen surface for the selected document.

Sections:

- title and metadata
- Markdown description preview
- subtasks
- related files
- accord/assignment
- validation commands/results
- review status and notes
- event timeline
- raw file path

Detail view should make PM validation easy:

```text
Actions: [accept] [request changes] [complete] [edit] [copy id]
```

### 2. Review queue

A dedicated filtered list showing items needing attention:

- accord delivered
- review pending
- validation failed
- blocked items
- accepted but not completed

This should answer: “What needs me?” without imposing hard-coded workflow sections in v0. Sorting should start simple, such as priority first, then most recently updated or delivered.

### 3. Logs view

A first-class completed-work browser.

Logs should show:

- completed timestamp
- summary
- files changed
- reviewer
- validation result
- accord status/evidence
- original item body
- event timeline

Actions:

- search logs
- inspect completion details
- copy summary
- open files changed

Deferred log actions:

- restore/reopen
- permanently delete only with strong confirmation

### 4. Rules view

A view for project rules:

- `always`
- `never`
- `prefer`
- `context`

Actions:

- add rule
- edit rule
- delete rule
- copy rule
- maybe promote decision to rule later

### 5. Decisions view

The TUI should allow browsing and managing `decision` documents outside normal task flow.

Decision documents do not have a v0 lifecycle field. The Decisions view should show `type: decision` documents and their Markdown body/metadata without inventing separate decision states.

## Layout modes

### Wide layout

For terminals >= ~120 columns:

```text
┌ Project title ──────────────── health/status/search ┐
├ Board | Review | Logs | Rules | Decisions ──────────┤
│ todo             in-progress          review         │
│ ┌───────────┐    ┌───────────┐       ┌───────────┐  │
│ │ cards     │    │ cards     │       │ cards     │  │
│ └───────────┘    └───────────┘       └───────────┘  │
├──────────────── selected detail / status ───────────┤
│ key hints / command mode / status messages           │
└──────────────────────────────────────────────────────┘
```

### Medium layout

For ~80-119 columns:

- state tabs at top
- single list for selected state
- right or lower detail pane if enough room

### Narrow layout

For ~50-79 columns:

- stacked global list grouped by state
- detail opens full-screen or as expandable cards
- no horizontal board assumptions

### Tiny terminal behavior

For terminals below minimum:

- show clear minimum size message
- avoid panics or corrupt terminal state

## Progress and health widgets

Do not compute progress from a `done` column.

Useful metrics:

- in-progress count
- review count
- blocked count
- delivered needing review
- completed today/week from logs/events
- accord statuses
- validation failures
- stale active items

Potential header:

```text
My Project  in-progress 4 · review 3 · blocked 1 · completed this week 7
```

Optional progress bars:

- epic progress: completed children / total children
- review queue: accepted / delivered
- validation: passed / total delivered
- decision/review progress if useful

## Theming

Theme support is required in MVP, not a later polish task.

### Built-in themes

Suggested built-ins:

- `default-dark`
- `default-light`
- `rose-pine`
- `catppuccin-mocha`
- `gruvbox-dark`
- `nord`
- `terminal` / no-truecolor fallback

### Theme file

V0 theme files use TOML. The intended full loading order remains built-in defaults, user theme files, then workspace theme override.

Config paths:

```text
~/.config/tandem/themes/*.toml   # planned user theme discovery
.tandem/theme.toml               # implemented workspace override
```

Current implementation starts from the built-in `default-dark` palette and applies `.tandem/theme.toml` if present. The parser intentionally accepts only simple TOML-style `key = "color"` entries plus section headers; it supports truecolor hex strings (`"#RRGGBB"` and `"#RGB"`) and terminal color names. Unknown keys and invalid colors are reported as non-fatal TUI status warnings.

Implemented keys:

```toml
name = "rose-pine-custom"

[colors]
background = "#191724"
panel = "#1f1d2e"
text = "#e0def4"
muted = "#6e6a86"
accent = "#c4a7e7"
success = "#9ccfd8"
warning = "#f6c177"
error = "#eb6f92"
border = "#403d52"
selected_bg = "#26233a"
selected_fg = "#e0def4"

[priority]
critical = "#eb6f92"
high = "#f6c177"
medium = "#31748f"
low = "#6e6a86"
none = "#6e6a86"

[badges.accord]
ready = "#f6c177"
claimed = "#31748f"
delivered = "#c4a7e7"
accepted = "#9ccfd8"
rework = "#ebbcba"
failed = "#eb6f92"
blocked = "#eb6f92"
unknown = "#6e6a86"

[badges.review]
not-ready = "#6e6a86"
pending = "#f6c177"
accepted = "#9ccfd8"
changes-requested = "#ebbcba"
rejected = "#eb6f92"
failed = "#eb6f92"
unknown = "#6e6a86"
```

`NO_COLOR=1` or `TANDEM_NO_COLOR=1` selects the terminal/no-color fallback. User theme discovery and named built-ins beyond `default-dark` remain planned.

### Theme requirements

- Support truecolor terminals.
- Support 256-color fallback where possible.
- Support no-color mode.
- Keep semantic color names separate from concrete colors.
- Make priority and status badges configurable.
- Avoid relying only on color; include glyphs/text for status.

## Mouse support

Mouse support should be built into the event model.

Required interactions:

- click tabs/views
- click cards to select
- double-click or enter/click to expand detail
- scroll lists with mouse wheel
- click action buttons in detail/review/log views
- click column/state picker in move mode
- click confirmation buttons

Deferred interactions:

- drag card between states
- drag to reorder within a state
- right-click/context menu
- mouse text selection compatibility toggle

### Hit map architecture

During render, widgets register interactive regions:

```rust
struct HitRegion {
    rect: Rect,
    action: UiAction,
    z_index: u16,
    label: String,
}
```

Mouse events resolve against the topmost matching region.

This avoids scattering coordinate math through the app.

## Keyboard model

Support both vim-like and conventional keys.

Global:

| Key | Action |
| --- | --- |
| `q` | quit |
| `?` | help |
| `:` | command palette / command line |
| `/` | search current view |
| `r` | reload |
| `1..5` | switch major view: Board, Review, Logs, Rules, Decisions |
| `tab` / `shift-tab` | next/previous section |
| `esc` | close modal/clear filter |

Navigation:

| Key | Action |
| --- | --- |
| `j/k` or arrows | move selection |
| `h/l` or left/right | move column/tab |
| `g/G` | top/bottom |
| `ctrl-d/u` | half-page down/up |
| `enter` | expand/open |

Work actions:

| Key | Action |
| --- | --- |
| `a` | quick-add task in the selected/default configured state (current slice) |
| `n` | new item quick add (planned keymap may be reconciled with `a`) |
| `N` | new item in editor |
| `e` | edit selected item in `$EDITOR` |
| `m` | move/change state |
| `p` | change priority |
| `A` | accord action menu (assign/claim/deliver; planned) |
| `v` | validation/review action menu |
| `c` | complete/archive, if allowed |
| `R` | reopen/restore in logs, if enabled after v0 |
| `d` | delete with confirmation |
| `y` | copy ID/link |

V0 uses fixed default keybindings. Keymap configuration is deferred until after the first MVP.

## Command palette

The command palette should expose every action so users do not have to memorize keys.

Examples:

```text
:new task
:move review
:complete
:accord deliver
:review accept
:log search theme system
:open file
:copy id
:theme catppuccin-mocha
```

## Search and filters

Search should support plain text and structured filters.

Examples:

```text
theme
state:review
accord:delivered
review:pending
priority:high tag:tui
assignee:pi blocked:true
file:src/theme.rs
```

Saved views should be possible via workspace config later:

```yaml
views:
  needs-review:
    query: "state:review OR accord:delivered"
  mine:
    query: "assignee:ivan OR assignee:pi"
```

## Markdown rendering

The first TUI MVP should support styled basics:

- headings
- bullet and numbered lists
- code fences as visibly distinct blocks
- inline code and emphasis
- links rendered as readable text/URLs

Tables, images, syntax highlighting, and advanced Markdown blocks are deferred.

## Editing model

The TUI should support two editing styles:

1. Quick inline forms for common fields.
2. `$EDITOR` for full Markdown/frontmatter editing.

Inline edits should call core mutation APIs that preserve unknown fields and minimize diffs.

External editor flow must:

- temporarily restore terminal state
- suspend mouse/raw mode
- wait for editor
- resume TUI cleanly
- reload changed files
- report parse/validation errors clearly

## File watcher and reload

The app should watch:

- workspace config
- `board/`
- `logs/`
- `events.jsonl`
- theme files

Theme config loading order is built-in defaults first, then user TOML theme files from `~/.config/tandem/themes/*.toml`, then workspace override `.tandem/theme.toml`. Workspace config wins when settings conflict.

Hot reload behavior:

- debounce changes
- preserve selection when possible
- show reload flash/status
- detect selected item deletion/move
- surface parse errors without crashing

## Minimal-diff write behavior

The CLI and TUI should treat Tandem files as hand-written documents, not generated state blobs. Mutations should preserve as much source text as practical while still enforcing required structure.

### Source preservation model

- Parse each Markdown document as three logical regions: opening frontmatter delimiter, frontmatter source, and Markdown body source.
- Keep the raw source for both frontmatter and body alongside any typed projection used by commands or views.
- Update only touched frontmatter fields where possible instead of serializing the full document from an in-memory object.
- Preserve unknown frontmatter fields exactly unless the user edits or removes them directly.
- Preserve frontmatter field order, comments, blank lines, and scalar style as much as practical. If a localized patch cannot safely preserve formatting, prefer a clear error or narrowly scoped rewrite over a whole-document rewrite.
- Preserve the Markdown body byte-for-byte unless the command or TUI action explicitly edits the body.
- `$EDITOR` flows may replace the full edited document because the user directly controls that edit; command-driven mutations should still use targeted patches.

### Command mutation coverage

These v0 command families mutate files and must follow the minimal-diff behavior:

- `tdm init`: creates new workspace files; no prior source exists, but generated files should be stable and readable.
- `tdm add`: writes one new task file and updates only required workspace/event state.
- `tdm move`: updates only an active task document's `state` and mutation timestamp fields that actually change.
- `tdm complete`: moves the document to logs, adds nested `completion` metadata, removes active-only fields only when required, and appends a separate event.
- `tdm accord ...`: updates only the `accord` subtree plus the task `updatedAt`; claim/deliver timestamps are written inside `accord`.
- `tdm rules add|edit|delete`: patches only the relevant rule category in workspace frontmatter.
- `tdm decision add`: writes one new decision file and appends a creation event.
- TUI quick edits and action buttons must call the same mutation behavior as CLI commands.

### Writes, timestamps, and events

- Use atomic writes for document rewrites: write a temp file in the same directory, flush it, then replace the target path.
- Do not leave temp files behind after successful writes; on failure, leave the original target unchanged and report the temp path only if cleanup fails.
- Detect concurrent edits before writing. A command or TUI action should compare the current file metadata/content identity with the snapshot it parsed; if the file changed, reload and revalidate before applying the mutation.
- Update `updatedAt` only for real mutations. Do not touch timestamps for read commands, no-op commands, failed validation, or unchanged writes.
- Append lifecycle events separately from document rewrites. The event append should not require reserializing the changed document.
- Event names must use Tandem-native domains, for example `task.created`, `task.moved`, `task.completed`, `decision.created`, `accord.delivered`, `review.updated`, and `rules.updated`.
- If a document mutation succeeds but event append fails, report the failure clearly. The implementation should either roll back when safe or surface a repair instruction; silently dropping the event is not acceptable.

### Error handling for writes

- If frontmatter or document structure cannot be parsed, do not attempt a mutation. Report the file path and the most specific location/field available.
- If validation fails, do not write partial changes. Warnings may be shown without blocking when the protocol marks them as warnings.
- If a minimal patch cannot be applied because the source changed or the target field is ambiguous, reload and retry once; if still ambiguous, fail with a clear message rather than rewriting unrelated fields.
- TUI write failures should leave UI state consistent with disk and show a status/error panel. Do not optimistically keep mutations that failed on disk.

## Implementation boundaries

The current implementation layout is a single Rust binary crate in `tandem-tui/` that builds `tdm`. It uses manual CLI parsing, raw-source mutation patches, the approved `yaml-rust2` dependency for frontmatter reads, and Ratatui/crossterm for the first TUI event loop and rendering layer. Do not assume or introduce a root Rust workspace, a multi-crate layout, a standalone shared implementation package, or a CLI parsing dependency without an explicit decision.

The behavioral boundaries should stay clear:

### Protocol behavior responsibilities

- discover `.tandem/` workspaces
- parse config and documents
- expose typed projections for commands/views
- preserve raw documents for minimal patches
- list/filter/query work documents
- mutate fields/states/accords/reviews
- complete/archive and any later reopen behavior
- append events

### CLI responsibilities

- expose scriptable `tdm` commands
- map command inputs to protocol behavior
- provide predictable human-readable output and `--json` structured output for all read commands
- report clear errors and policy failures
- launch the TUI through `tdm tui` in v0

### TUI responsibilities

- app state
- event loop
- rendering widgets
- keyboard/mouse mapping
- command palette
- forms/modals
- theme loading
- file watching integration

## Possible dependency areas (not settled)

Potential implementation dependencies should be chosen deliberately and kept minimal. The current CLI keeps manual argument parsing and uses `yaml-rust2` for frontmatter/config/document read parsing. The first TUI slice uses Ratatui for rendering and crossterm for terminal input/backend handling.

Need to choose later:

- Whether to keep manual CLI parsing or approve a CLI parser crate after v0 command behavior stabilizes.

- Serialization/frontmatter/event parsing strategy.
- Whether to keep the current no-dependency workspace theme parser or later approve a fuller TOML parser for user theme discovery at `~/.config/tandem/themes/*.toml`.
- File watching strategy for the first TUI MVP.
- ID/timestamp helper strategy, if helpers are needed.
- Whether to keep the direct crossterm event loop or introduce a thin internal event abstraction as the TUI grows.
- Text input widgets vs simple custom forms.
- Markdown rendering strategy for the locked styled-basics behavior.

## App state sketch

```rust
enum View {
    Board,
    Review,
    Logs,
    Rules,
    Decisions,
}

enum Mode {
    Browse,
    Search,
    Command,
    Move,
    EditForm,
    Confirm,
    Help,
}

struct AppState {
    workspace: WorkspaceState,
    view: View,
    mode: Mode,
    selection: SelectionState,
    filters: FilterState,
    theme: Theme,
    status: Option<StatusMessage>,
    hit_map: HitMap,
}
```

## UI actions

All input should resolve to actions:

```rust
enum UiAction {
    SwitchView(View),
    SelectNext,
    SelectPrev,
    OpenSelected,
    MoveSelected { state: String },
    CompleteSelected,
    AccordClaim,
    AccordDeliver,
    AccordAccept,
    AccordRework,
    AccordBlock,
    AccordFail,
    ReviewAccept,
    ReviewRequestChanges,
    OpenCommandPalette,
    Search(String),
    Reload,
    Quit,
}
```

This keeps keyboard, mouse, and command palette behavior consistent.

## Review workflow UX

Happy path:

1. Agent/human delivers item.
2. Item appears in Review queue with `[A:delivered]`.
3. User opens detail.
4. TUI shows deliverables, files changed, validation commands/results, summary.
5. User selects:
   - accept accord
   - request changes
   - run validation
   - complete/archive
6. Completion moves item to logs and appends event.

Important: accepted accord and completed work are not the same thing. The TUI should make this distinction visible.

## Completion UX

When pressing complete/archive:

- show summary form
- show policy checks
- show validation status
- ask for reviewer confirmation if required
- allow files changed list
- then archive to logs

Example:

```text
Complete task-7?

Summary: [ Theme system implemented and tested                    ]
Validation: cargo test passed, cargo clippy passed
Accord: accepted by ivan
Files: 3 changed

[Complete] [Cancel]
```

## Logs UX

Logs are not a trash can. They are a memory/audit surface.

Expanded log item should show:

```text
task-7 Implement Ratatui theme system
Completed: 2026-06-26 15:00 by ivan
Summary: Theme loading, built-in palettes, runtime style mapping.
Validation: passed
Files changed:
  src/tui/theme.rs
  src/tui/app.rs
Accord:
  assignee: pi
  status: accepted
Timeline:
  12:00 created
  12:05 claimed
  13:30 delivered
  15:00 completed
```

## Error handling

The CLI and TUI should never silently corrupt project files.

When a file has invalid frontmatter or document structure:

- keep the app open
- show error panel
- identify file and line/field if possible
- allow opening in editor
- allow reload after fix

For failed writes:

- show precise error
- do not update UI optimistically unless recoverable
- log operation failure to status area

## Testing strategy

Exact test tooling is open, but planned coverage should include:

CLI/protocol behavior:

- workspace discovery
- read command output, including `--json` for all read commands
- add/move/complete/log/search/accord/rules/decision flows
- minimal-diff document mutation behavior
- event ledger append behavior
- completion behavior and any later reopen/restore behavior

TUI behavior:

- widget/screen snapshot tests if supported by the chosen stack
- input-to-action tests
- mouse hit-map tests
- layout breakpoint tests
- theme parsing and rendering tests
- board mutation flows from inside the TUI

Manual smoke:

- small terminal
- wide terminal
- no-color terminal
- mouse scroll/click
- external editor open/return
- file changed externally while TUI is open

## MVP phases

### Phase 0: v0 CLI design lock

- Specify options and output shape for the locked v0 command families.
- Define `--json` behavior for all read commands.
- Define warnings and policy checks for `tdm complete`.
- Define detailed options and output for `tdm decision list|show|add`.
- Keep implementation layout inside `tandem-tui/` and dependency choices open until coding begins.

### Phase 1: v0 CLI implementation

- Implement in Rust inside `tandem-tui/`.
- Implement workspace discovery and document reading.
- Implement v0 read commands: `list`, `show`, `log list`, `log show`, `log search`, `search`, read-oriented `rules`, and read-oriented `decision` operations.
- Implement v0 mutations: `init`, `add`, `move`, `complete`, `accord`, `rules`, and decision operations.
- Preserve unknown fields and minimize document rewrites.

### Phase 2: First TUI MVP

- Launch through `tdm tui`.
- Started with a Ratatui/crossterm shell that renders top-level Board, Review, Logs, Rules, and Decisions tabs; Review/Logs/Rules/Decisions currently have read-only placeholders/counts.
- Board renders active board documents with navigation, details, reload, help, safe quit, quick-add via `a`, move-state mutation via `H`/`L`, built-in `default-dark` theme styling, and workspace `.tandem/theme.toml` color overrides.
- Render full Review, Logs, Rules, and Decisions workflows on top of the existing view shell.
- Include board mutations immediately: add, move state, edit, complete, accord actions, rules actions, and supported decision actions.
- Include built-in theme support and user-selectable theme loading.
- Include mouse selection, scrolling, tab switching, and action-button clicks enabled by default.
- Exclude drag/drop from v0.
- Render Markdown with styled basics.
- Hot reload file changes and surface parse/write errors safely.

### Phase 3: TUI polish and post-MVP features

- Configurable keymap.
- Saved filters/views.
- Drag/reorder if desired.
- Richer Markdown rendering if needed.
- Additional integrations only after v0 CLI/TUI workflows are stable.

## Open questions

All previously listed CLI/TUI policy questions are now resolved. Remaining existing-work focus is TUI implementation, tracked in `todo.md`; CLI changes should be explicit new features or bug fixes.
