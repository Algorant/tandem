# Tandem Protocol Spec Draft

Status: draft  
Date: 2026-06-26  
Working name: Tandem

This document sketches the Tandem protocol: a Brainfile-inspired file format that keeps Brainfile's useful shape, adapts the language for Tandem, and folds in local v3 improvements around review, complete/archive, and first-class logs.

The protocol is the spec/source of truth for Tandem's local-first project coordination files. It is not an implementation package or crate layout.

## Baseline inputs

Tandem protocol work should reconcile these inputs:

- Live Brainfile protocol shape: `.brainfile/brainfile.md`, `board/`, `logs/`, Markdown files with YAML frontmatter, rules, custom types, parent links, subtasks, contracts, logs, and CLI/tool operations.
- Brainfile public references: `https://github.com/brainfile/protocol` and `https://brainfile.md/reference/protocol`.
- Local Brainfile v3 proposal: `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md`.

Brainfile is a design reference, not a v0 compatibility target. Tandem should keep the useful basic shape, but it has no required Brainfile importer, migrator, or legacy discovery mode in v0.

## Locked v0 protocol decisions

- Tandem is Brainfile-inspired, not a Brainfile compatibility layer.
- No Brainfile importer or migration command is required in v0.
- Canonical workflow field names are `state` on documents and `states` in workspace config.
- Default active states are `todo`, `in-progress`, and `review`.
- New task items use `type: task` and sequential IDs such as `task-1`.
- First-class document types are `task` and `decision`.
- Custom document types are allowed in config only; v0 has no type-management CLI.
- The work-agreement object is `accord`.
- Accord statuses are `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, and `blocked`.
- Rules are structured objects with stable IDs: `{ id, rule, source? }`.
- `parentId`, `blockers`, and `references` may point to any Tandem document by ID.
- Subtask IDs are parent-based sequential IDs, such as `task-1-1`.
- Completion warns about missing accepted review or accepted accord, but allows completion in v0.
- Protocol version for the first v0 draft is `0.1.0`.
- Events live at `.tandem/events.jsonl`.
- Event payloads are minimal audit records in v0: require `ts`, `event`, `id`, and `summary`; defer typed per-event payload schemas.
- Completed logs are archived Markdown documents in `.tandem/logs/`; those documents are the source of truth for completed work. Events enrich timeline and audit views.
- Validation/lint is built-in structural validation only in v0.
- Schemas and fixtures are not part of v0.
- Validation severity is strict for structure and core references: invalid/missing required structure and unresolved `parentId`/`blockers` are errors; unresolved related `references`/rule sources and completion-policy issues are warnings.
- Decision documents do not need a lifecycle field in v0.

## Naming model

- Product/protocol: **Tandem**
- Repository: `tandem`
- Protocol data directory: `.tandem/`
- Protocol config file: `.tandem/tandem.md`
- CLI binary: `tdm`
- CLI/TUI work directory in this monorepo: `tandem-tui/`
- User-facing CLI: `tdm`; reserve `td` for future/internal tool prefixes
- Work-agreement object replacing Brainfile's `contract` concept: `accord`

`tdm` is intentionally short enough for daily terminal use while still reading as Tandem. `td` is reserved for future/internal tool prefixes unless explicitly revisited.

## Goals

- Keep project coordination state in readable, editable Markdown files.
- Make the on-disk format easy to diff, review, merge, and repair.
- Treat active tasks, decisions, reviews, accords, rules, and completed logs as first-class project artifacts.
- Support both human project management and AI-agent execution workflows.
- Avoid a redundant persistent completed board state when completed work already lives in logs.
- Preserve unknown fields and minimize rewrites so tools do not destroy user edits.
- Make the protocol implementable by multiple tools, not only one CLI/TUI.

## Non-goals

- Recreating Brainfile exactly under a new name.
- Shipping a Brainfile importer or migrator in v0.
- Requiring a database for normal use.
- Hiding the source of truth behind opaque binary state.
- Forcing one task methodology such as Scrum, Kanban, GTD, or Linear-style issue tracking.
- Making agent accords mandatory for every task.
- Settling implementation layout, Rust crates, schema directories, fixtures, CI, or dependency choices.
- Adding schema or fixture directories in v0.

## Core model

The protocol is a project-local directory containing:

```text
.tandem/
├── tandem.md          # workspace/board configuration and project context
├── board/             # active/current Tandem documents
│   ├── task-1.md
│   └── decision-1.md
├── logs/              # completed/archived Markdown documents
│   └── task-1.md
└── events.jsonl       # append-only lifecycle events
```

Tandem examples use the project-local directory and config file:

```text
.tandem/
.tandem/tandem.md
```

## Brainfile design reference

Brainfile gets several important things right and is the design baseline:

- File-based source of truth.
- Markdown files with YAML frontmatter.
- One active task per file.
- A separate completed-work area.
- Project rules visible to humans and agents.
- Custom document types.
- Agent-oriented assignment/agreement metadata.
- CLI and tool friendliness.

Tandem preserves those ideas while changing the parts that feel underdeveloped or awkward in practice. Changes from Brainfile should be deliberate and documented, but v0 does not require legacy file discovery or conversion tooling.

## Key changes from Brainfile

### 1. Completion is an action, not a board state

Default lifecycle:

```text
todo → in-progress → review → complete/archive → logs
```

Completed work belongs in `.tandem/logs/` and is enriched by `.tandem/events.jsonl`.

Projects may define additional active states, but `todo`, `in-progress`, and `review` are the v0 defaults. A persistent completed board state is not part of the default workflow.

### 2. Separate human workflow state from agent accord state

Brainfile's contract status and task column can drift or overlap conceptually. Tandem makes the layers explicit:

```yaml
state: review              # human/project workflow state
accord:
  status: delivered        # agent/human agreement state
review:
  status: pending          # validation/review state
```

This allows a task to be in human `review` while the accord is `delivered`, `accepted`, or `rework`.

### 3. Replace Brainfile's contract concept with accord

Brainfile's `contract` term is technically clear but feels legalistic and one-sided. Tandem uses `accord` for the explicit human/agent work agreement: the shared brief, deliverables, constraints, validation expectations, evidence, and acceptance state.

`accord` has the right tone: collaborative, mutual, and workflow-oriented without sounding like a legal document.

### 4. Logs are first-class

Logs should not be a thin archive folder. They should support PM review, postmortems, search, audit, and accord history.

A completed task should expose:

- completion summary
- completed timestamp
- files changed
- validation results or recorded evidence
- accord evidence
- reviewer notes
- related events
- original Markdown body

### 5. IDs are simple and sequential in v0

New v0 tasks use sequential IDs:

```yaml
id: task-1
type: task
```

Suggested filename:

```text
task-1-implement-theme-system.md
```

The ID is canonical. A readable filename suffix may change without changing the document identity.

Decision documents use the same pattern with their own prefix:

```yaml
id: decision-1
type: decision
```

Subtasks use parent-based sequential IDs:

```yaml
subtasks:
  - id: task-1-1
    title: Define theme behavior
    completed: false
```

Custom document types may define their own `idPrefix` in workspace config, but v0 does not include type-management commands.

## Discovery

Tools should discover a Tandem workspace in this order:

1. `.tandem/tandem.md`
2. `tandem.md` in the repository root, for simple/single-file compatibility within Tandem itself
3. no legacy Brainfile discovery paths in v0

Discovery should stop at repository boundaries unless explicitly told to search parent directories.

## Board/workspace config

Example:

```markdown
---
protocolVersion: 0.1.0
title: My Project
states:
  - id: todo
    title: To Do
  - id: in-progress
    title: In Progress
  - id: review
    title: Review
completion:
  action: archive-to-logs
  warnIfReviewNotAccepted: true
  warnIfAccordNotAccepted: true
types:
  task:
    idPrefix: task
    completable: true
  decision:
    idPrefix: decision
    completable: false
  bug:
    idPrefix: bug
    completable: true
rules:
  always:
    - id: rule-1
      rule: Preserve IDs.
      source: decision-1
  never:
    - id: rule-2
      rule: Commit secrets.
  prefer:
    - id: rule-3
      rule: Make small focused changes.
  context:
    - id: rule-4
      rule: This project uses Tandem protocol files for coordination.
agent:
  instructions:
    - Preserve IDs.
    - Preserve unknown fields.
    - Prefer minimal frontmatter patches over full rewrites.
---

# My Project

Human-readable project context goes here.
```

### Config fields

| Field | Required | Purpose |
| --- | --- | --- |
| `protocolVersion` | yes | Protocol version. |
| `title` | yes | Display name. |
| `states` | yes | Human workflow states. |
| `completion` | no | Completion/archive warning policy. |
| `types` | no | First-class and custom document type configuration. Custom types are config-only in v0. |
| `rules` | no | Structured project rules for humans and agents. |
| `agent` | no | Agent-specific operating guidance. |
| `theme` | no | Optional TUI theme preference. |
| `views` | no | Optional saved filters/views. |

The v0 spec uses built-in structural validation. Schema URLs, schema files, fixtures, and schema-management commands are deferred out of v0.

## Task document

Example:

```markdown
---
id: task-1
type: task
title: Implement Ratatui theme system
state: in-progress
priority: high
effort: medium
tags: [tui, rust]
assignee: pi
parentId: decision-1
references:
  - decision-2
relatedFiles:
  - src/tui/theme.rs
blockers: []
createdAt: 2026-06-26T12:00:00Z
updatedAt: 2026-06-26T12:20:00Z
accord:
  status: claimed
  assignee: pi
  claimedAt: 2026-06-26T12:05:00Z
  deliverables:
    - type: file
      path: src/tui/theme.rs
      description: Theme parser and runtime palette mapping
      required: true
  validation:
    commands:
      - cargo test
      - cargo clippy --all-targets
  constraints:
    - Do not introduce a database dependency.
review:
  status: not-ready
subtasks:
  - id: task-1-1
    title: Define theme behavior
    completed: false
---

## Description

Build a user-configurable theme layer for the Ratatui TUI.

## Notes

Freeform notes stay in Markdown and should not be destroyed by tools.
```

### Task fields

| Field | Required | Purpose |
| --- | --- | --- |
| `id` | yes | Stable canonical identifier such as `task-1`. |
| `type` | no | Defaults to `task` for new task documents. |
| `title` | yes | Display title. |
| `state` | yes for active tasks | Human workflow state. Defaults are `todo`, `in-progress`, and `review`. |
| `priority` | no | `low`, `medium`, `high`, `critical`, or project-defined. |
| `effort` | no | `trivial`, `small`, `medium`, `large`, `xlarge`, or project-defined. |
| `tags` | no | Filtering/grouping. |
| `assignee` | no | Human or agent currently responsible. |
| `parentId` | no | Parent Tandem document ID. May point to any document type. |
| `references` | no | Related Tandem document IDs. May point to any document type. |
| `relatedFiles` | no | Project paths relevant to the task. |
| `blockers` | no | Tandem document IDs blocking this item. May point to any document type. |
| `accord` | no | Agent/human work agreement. |
| `review` | no | Review and validation state. |
| `subtasks` | no | Lightweight checklist items with parent-based sequential IDs. |
| `createdAt` | no | Creation timestamp. |
| `updatedAt` | no | Last mutation timestamp. |
| `completedAt` | logs only | Completion timestamp. |

## Decision document

Decision documents are first-class v0 documents. They capture durable project, product, or architecture decisions and may be referenced by tasks, blockers, rules, and other decisions.

Example:

```markdown
---
id: decision-1
type: decision
title: Use accord vocabulary for work agreements
createdAt: 2026-06-26T12:00:00Z
updatedAt: 2026-06-26T12:10:00Z
references:
  - task-1
---

# Decision

Use `accord` for the collaborative work-agreement object.

## Rationale

The term is less legalistic than Brainfile's contract terminology and better matches Tandem's collaborative tone.
```

Required v0 decision fields are `id`, `type: decision`, and `title`. Decision documents do not need a lifecycle field in v0; their durable decision content lives in frontmatter metadata plus the Markdown body.

## Accord model

Working term: `accord`.

```yaml
accord:
  status: ready      # ready | claimed | delivered | accepted | rework | failed | blocked
  assignee: pi
  claimedAt: 2026-06-26T12:05:00Z
  deliveredAt: null
  summary: null
  deliverables:
    - type: file
      path: src/main.rs
      description: Main implementation
      required: true
  validation:
    commands:
      - cargo test
  evidence:
    - type: command
      command: cargo test
      status: passed
      summary: 42 tests passed
  constraints:
    - No network calls during tests.
  outOfScope:
    - Web UI.
```

### Accord lifecycle

```text
ready → claimed → delivered → accepted
                 ↘ rework → claimed/delivered
                 ↘ failed
                 ↘ blocked
```

Suggested relationship to task state:

| Accord status | Suggested task state |
| --- | --- |
| `ready` | `todo` |
| `claimed` | `in-progress` |
| `delivered` | `review` |
| `accepted` | `review` until completion/archive |
| `rework` | `in-progress` or `review`, depending on project preference |
| `failed` | any state plus failure indicator |
| `blocked` | any state plus blocked indicator |

The protocol should preserve the distinction between accord state and human workflow state. TUI/CLI tools should make misalignment visible rather than silently hiding it.

## Review model

```yaml
review:
  status: pending     # not-ready | pending | accepted | changes-requested | rejected
  reviewer: ivan
  requestedAt: 2026-06-26T13:00:00Z
  decidedAt: null
  notes: []
```

Review is separate from accord. A delivered accord may still need human acceptance, additional validation, or polish.

In v0, completion should warn if `review.status` is not `accepted`, but it should still allow completion.

## Completion and logs

Completion is a mutation that:

1. Runs built-in structural validation.
2. Warns when review or accord acceptance is missing.
3. Appends a completion event to `.tandem/events.jsonl`.
4. Sets `completedAt` and `completion` metadata on the document.
5. Moves the document from `.tandem/board/` to `.tandem/logs/`.

Example completed document frontmatter:

```yaml
id: task-1
type: task
title: Implement Ratatui theme system
completedAt: 2026-06-26T15:00:00Z
completion:
  summary: Theme loading, built-in palettes, and runtime style mapping implemented.
  filesChanged:
    - src/tui/theme.rs
    - src/tui/app.rs
  validation:
    status: passed
    commands:
      - command: cargo test
        status: passed
  reviewer: ivan
accord:
  status: accepted
```

Archived Markdown documents in `.tandem/logs/` are the source of truth for completed work. Events enrich timeline, audit, and search views, but tools should not need events to reconstruct the current board or completed-log corpus.

## Events

`.tandem/events.jsonl` should be append-only and machine-readable.

V0 event records are minimal audit-only records. Required fields:

| Field | Required | Purpose |
| --- | --- | --- |
| `ts` | yes | Event timestamp. |
| `event` | yes | Event name such as `task.created`, `accord.claimed`, or `task.completed`. |
| `id` | yes | Tandem document ID that the event is about. |
| `summary` | yes | Human-readable audit summary. |

Tools may include optional top-level fields such as `actor` or `details`, but v0 does not require typed per-event payload schemas.

Example events:

```jsonl
{"ts":"2026-06-26T12:00:00Z","event":"task.created","id":"task-1","summary":"Created task: Implement Ratatui theme system","actor":"ivan"}
{"ts":"2026-06-26T12:05:00Z","event":"accord.claimed","id":"task-1","summary":"pi claimed the accord"}
{"ts":"2026-06-26T13:30:00Z","event":"accord.delivered","id":"task-1","summary":"Initial implementation ready"}
{"ts":"2026-06-26T15:00:00Z","event":"task.completed","id":"task-1","summary":"Completed and archived to logs","actor":"ivan"}
```

Events should never be required to reconstruct the current board. They provide audit/history and power richer logs.

## Rules

Rules remain centralized in the workspace config and use structured objects:

```yaml
rules:
  always:
    - id: rule-1
      rule: Write tests for new features.
      source: decision-1
  never:
    - id: rule-2
      rule: Commit secrets.
  prefer:
    - id: rule-3
      rule: Small focused changes.
  context:
    - id: rule-4
      rule: This project uses Tandem for local-first coordination.
```

Rules are for humans and agents. Agents should read them before starting work. `source` is optional and may point to any Tandem document ID.

## Document types

First-class v0 types:

- `task` — normal task/work item. Completable by default.
- `decision` — durable project, product, or architecture decision. Not completable by default.

Custom types are allowed only through workspace config in v0:

```yaml
types:
  task:
    idPrefix: task
    completable: true
  decision:
    idPrefix: decision
    completable: false
  bug:
    idPrefix: bug
    completable: true
```

A custom type may define an `idPrefix` and whether documents of that type are completable. v0 does not include commands for creating, editing, or managing type definitions.

## Validation and lint

V0 validation/lint is built-in structural validation only.

Severity policy:

- **Errors:** invalid frontmatter, missing required fields, duplicate document IDs, unknown active task states, unknown accord/review statuses, invalid structured rules, invalid subtask ID shape, unresolved `parentId`, and unresolved `blockers`.
- **Warnings:** unresolved related `references`, unresolved rule `source` links, missing accepted review/accord during completion, and non-canonical but recoverable metadata.

Structural checks should cover at least:

- workspace config parses and has `protocolVersion`, `title`, and `states`
- configured state IDs are unique
- default states `todo`, `in-progress`, and `review` exist unless the project intentionally overrides defaults
- document frontmatter parses
- document IDs are unique
- document `type` is first-class or configured as a custom type
- active task `state` exists in workspace `states`
- accord status is one of the canonical v0 statuses
- review status is one of the documented review statuses
- rules are structured objects with `id` and `rule`
- subtask IDs follow the parent-based sequential pattern
- `parentId`, `blockers`, and `references` target Tandem document IDs when present

V0 structural validation does not execute project validation commands, enforce remote schemas, run hooks, perform auth checks, or manage custom type definitions.

## Mutation rules for tools

Tooling must be careful. This is a core quality bar.

Tools should:

- Preserve document IDs.
- Preserve unknown fields.
- Preserve Markdown bodies.
- Prefer minimal frontmatter patches over whole-file rewrites.
- Avoid reordering fields unless explicitly formatting.
- Avoid touching unrelated files.
- Write timestamps only for real mutations.
- Use atomic writes where possible.
- Handle concurrent edits with file change detection.

Tools may build typed projections for querying and validation, but the raw Markdown document remains the source of truth.

## Protocol-facing CLI surface sketch

Using `tdm` as the working CLI binary name:

```text
tdm init
tdm list
tdm show <id>
tdm add --title ... --state todo
tdm move <id> --state review
tdm complete <id> --summary ...
tdm log list|show|search
tdm search <query>
tdm accord ready|claim|deliver|accept|rework|block|fail
tdm rules list|add|edit|delete
tdm decision list|show|add
tdm tui
```

This is protocol-facing command shape only. Detailed CLI/TUI behavior belongs in `../tandem-tui/` and must stay synchronized with protocol decisions.

## Brainfile design mapping/reference only

Tandem uses Brainfile as a design reference, not as a required conversion target. No v0 command is required for converting Brainfile boards.

Useful conceptual mappings for design discussions:

```text
Brainfile board config       → Tandem workspace config
.brainfile/brainfile.md      → .tandem/tandem.md
.brainfile/board/*.md       → .tandem/board/*.md
.brainfile/logs/*.md        → .tandem/logs/*.md
Brainfile column             → Tandem state
Brainfile contract concept   → Tandem accord
Brainfile archived task      → Tandem completed log document
ADR-style record             → Tandem decision document
```

## Open protocol questions

All previously listed v0 protocol decision questions are now resolved. Remaining protocol work is specification detail, tracked in `todo.md`.
