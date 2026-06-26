# Tandem Protocol Spec Draft

Status: draft  
Date: 2026-06-26  
Working name: Tandem

This document sketches a new protocol inspired by Brainfile's best ideas, but intentionally not limited to Brainfile's current naming, lifecycle, implementation choices, or TUI assumptions.

The goal is a durable, file-based project coordination format that works well for humans, AI agents, CLI tools, and a first-class terminal UI.

## Naming model

- Product/protocol: **Tandem**
- Repository: `tandem`
- Protocol data directory: `.tandem/`
- Protocol config file: `.tandem/tandem.md`
- CLI binary: `tdm`
- TUI work directory in this monorepo: `tandem-tui/`
- Future shorthand/prefix space: `td` or `tdm` for Pi extensions, tools, and integrations
- Work-agreement object replacing Brainfile `contract`: `accord`

`tdm` is intentionally short enough for daily terminal use while still reading as Tandem. `td` remains available as a broader prefix for future integrations.

## Goals

- Keep work state in readable, editable Markdown files.
- Make the on-disk format easy to diff, review, merge, and repair.
- Treat active work, reviews, accords, decisions, rules, and completed logs as first-class project artifacts.
- Support both human project management and AI-agent execution workflows.
- Avoid a fake or redundant `done` column when completed work already lives in logs.
- Preserve unknown fields and minimize rewrites so tools do not destroy user edits.
- Make the protocol implementable by multiple tools, not only one CLI/TUI.

## Non-goals

- Recreating Brainfile exactly under a new name.
- Requiring a database for normal use.
- Hiding the source of truth behind opaque binary state.
- Forcing one task methodology such as Scrum, Kanban, GTD, or Linear-style issue tracking.
- Making agent accords mandatory for every task.

## Core model

The protocol is a project-local directory containing:

```text
.tandem/
├── tandem.md          # workspace/board configuration and project context
├── board/             # active work documents
│   ├── work_01j...md
│   └── decision_01j...md
├── logs/              # completed/archived work documents
│   ├── work_01j...md
│   └── decision_01j...md
└── events.jsonl       # append-only ledger of lifecycle events
```

Tandem examples use the project-local directory and config file:

```text
.tandem/
.tandem/tandem.md
```

## What to keep from Brainfile

Brainfile gets several important things right:

- File-based source of truth.
- Markdown files with YAML frontmatter.
- One document per active task/work item.
- A separate completed-work area.
- Project rules visible to humans and agents.
- Custom document types.
- Agent-oriented assignment/accord metadata.
- CLI and MCP/tool friendliness.

This protocol should preserve those ideas while changing the parts that feel underdeveloped or awkward in practice.

## Key changes from Brainfile

### 1. Completion is an action, not a column

Default lifecycle:

```text
backlog/todo → active/in-progress → review → complete/archive → logs
```

There should not be a default persistent `done` column. Completed work belongs in `logs/` and the event ledger.

A project may define a completion column for compatibility or personal preference, but it should not be the default and should not be required for progress metrics.

### 2. Separate human workflow state from agent accord state

Brainfile's `accord.status` and task `column` can drift or overlap conceptually. This protocol should make the layers explicit:

```yaml
state: review              # human/project workflow state
accord:
  status: delivered        # agent assignment state
review:
  status: pending          # validation/review state
```

This allows a task to be in human `review` while the agent accord is `delivered`, `accepted`, or `rework`.

### 3. Replace `contract` with `accord`

Brainfile's `contract` term is technically clear but feels legalistic and one-sided. Tandem should use `accord` for the explicit human/agent work agreement: the shared brief, deliverables, constraints, validation, evidence, and acceptance state.

`accord` has the right tone: collaborative, mutual, and workflow-oriented without sounding like a legal document.

### 4. Logs are first-class

Logs should not be a thin archive folder. They should support PM review, postmortems, search, audit, and agent accord history.

A completed item should expose:

- completion summary
- completed timestamp
- files changed
- validation results
- accord evidence
- reviewer notes
- related events
- original Markdown body

### 5. More robust IDs

Sequential IDs like `task-1` are readable, but fragile across branches and multiple agents. Prefer stable, low-collision IDs with optional readable slugs:

```yaml
id: work_01j2abcxyz9q...
slug: implement-ratatui-theme-system
```

Suggested filename:

```text
work_01j2abcxyz9q-implement-ratatui-theme-system.md
```

The ID is canonical. The slug is user-facing and may change.

## Discovery

Tools should discover a Tandem workspace in this order:

1. `.tandem/tandem.md`
2. `tandem.md` in the repository root, for simple/single-file compatibility
3. legacy import paths, if supported by migration tooling

Discovery should stop at repository boundaries unless explicitly told to search parent directories.

## Board/workspace config

Example:

```markdown
---
schema: https://example.invalid/tandem/v0/workspace.json
protocolVersion: 0.1.0
type: workspace
title: My Project
states:
  - id: backlog
    title: Backlog
  - id: todo
    title: To Do
  - id: active
    title: Active
  - id: review
    title: Review
completion:
  action: archive-to-logs
  requireReview: false
  requireAcceptedAccord: false
types:
  work:
    idPrefix: work
    completable: true
  epic:
    idPrefix: epic
    completable: true
  decision:
    idPrefix: decision
    completable: false
rules:
  always: []
  never: []
  prefer: []
  context: []
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
| `schema` | no | Schema URL for tooling validation. |
| `protocolVersion` | yes | Protocol version. |
| `type` | no | Should be `workspace`; default may be inferred. |
| `title` | yes | Display name. |
| `states` | yes | Human workflow states/columns. |
| `completion` | no | Completion/archive policy. |
| `types` | no | Custom document type configuration. |
| `rules` | no | Project rules for humans and agents. |
| `agent` | no | Agent-specific operating guidance. |
| `theme` | no | Optional TUI theme preference. |
| `views` | no | Optional saved filters/views. |

## Work document

Example:

```markdown
---
id: work_01j2abcxyz9q
slug: implement-ratatui-theme-system
type: work
title: Implement Ratatui theme system
state: active
priority: high
effort: medium
tags: [tui, rust]
assignee: pi
parentId: epic_01j2parent
relatedFiles:
  - crates/tui/src/theme.rs
blockedBy: []
createdAt: 2026-06-26T12:00:00Z
updatedAt: 2026-06-26T12:20:00Z
accord:
  status: claimed
  assignee: pi
  claimedAt: 2026-06-26T12:05:00Z
  deliverables:
    - type: file
      path: crates/tui/src/theme.rs
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
  - id: work_01j2abcxyz9q-1
    title: Define theme schema
    completed: false
---

## Description

Build a user-configurable theme layer for the Ratatui TUI.

## Notes

Freeform notes stay in Markdown and should not be destroyed by tools.
```

### Work fields

| Field | Required | Purpose |
| --- | --- | --- |
| `id` | yes | Stable canonical identifier. |
| `slug` | no | Human-readable filename/display slug. |
| `type` | no | Defaults to `work`. |
| `title` | yes | Display title. |
| `state` | yes for active docs | Human workflow state. |
| `priority` | no | `low`, `medium`, `high`, `critical`, or project-defined. |
| `effort` | no | `trivial`, `small`, `medium`, `large`, `xlarge`, or project-defined. |
| `tags` | no | Filtering/grouping. |
| `assignee` | no | Human or agent currently responsible. |
| `parentId` | no | Parent document ID. |
| `relatedFiles` | no | Project paths relevant to the work. |
| `blockedBy` | no | IDs blocking this item. |
| `accord` | no | Agent/human work agreement. |
| `review` | no | Review and validation state. |
| `subtasks` | no | Lightweight checklist items. |
| `createdAt` | no | Creation timestamp. |
| `updatedAt` | no | Last mutation timestamp. |
| `completedAt` | logs only | Completion timestamp. |

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

Suggested relationship to board state:

| Accord status | Suggested work state |
| --- | --- |
| `ready` | `todo` |
| `claimed` | `active` |
| `delivered` | `review` |
| `accepted` | `review` until completion/archive |
| `rework` | `active` or `review`, depending on project preference |
| `blocked` | any state plus blocked indicator |

The protocol should allow projects to configure this sync, but the TUI should show misalignment clearly.

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

## Completion and logs

Completion is a mutation that:

1. Validates policy requirements.
2. Appends a completion event to `events.jsonl`.
3. Sets `completedAt` on the document.
4. Removes active-only fields if desired, such as `state`.
5. Moves the document from `board/` to `logs/`.

Example completed document frontmatter:

```yaml
id: work_01j2abcxyz9q
type: work
title: Implement Ratatui theme system
completedAt: 2026-06-26T15:00:00Z
completion:
  summary: Theme loading, built-in palettes, and runtime style mapping implemented.
  filesChanged:
    - crates/tui/src/theme.rs
    - crates/tui/src/app.rs
  validation:
    status: passed
    commands:
      - command: cargo test
        status: passed
  reviewer: ivan
accord:
  status: accepted
```

## Event ledger

`events.jsonl` should be append-only and machine-readable.

Example events:

```jsonl
{"ts":"2026-06-26T12:00:00Z","event":"work.created","id":"work_01j2abcxyz9q","title":"Implement Ratatui theme system"}
{"ts":"2026-06-26T12:05:00Z","event":"accord.claimed","id":"work_01j2abcxyz9q","assignee":"pi"}
{"ts":"2026-06-26T13:30:00Z","event":"accord.delivered","id":"work_01j2abcxyz9q","summary":"Initial implementation ready"}
{"ts":"2026-06-26T15:00:00Z","event":"work.completed","id":"work_01j2abcxyz9q","reviewer":"ivan"}
```

Events should never be required to reconstruct the current board, but they should provide audit/history and power richer logs.

## Rules

Rules remain centralized in the workspace config:

```yaml
rules:
  always:
    - id: 1
      rule: Write tests for new features.
      source: decision_01j...
  never:
    - id: 1
      rule: Commit secrets.
  prefer:
    - id: 1
      rule: Small focused changes.
  context:
    - id: 1
      rule: Rust workspace with Ratatui TUI.
```

Rules are for humans and agents. Agents should read them before starting work.

## Document types

Default types:

- `work` — normal task/work item.
- `epic` — parent/container work.
- `decision` — architecture/product decision record.
- `note` — durable project note.

Types should be extensible:

```yaml
types:
  bug:
    idPrefix: bug
    completable: true
    schema: https://example.invalid/tandem/v0/bug.json
  experiment:
    idPrefix: exp
    completable: true
```

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

Suggested internal representation:

```rust
struct RawDocument {
    path: PathBuf,
    raw_frontmatter: String,
    typed_projection: Document,
    body: String,
}
```

The typed projection is for validation/querying. The raw document is for minimal-diff mutation.

## CLI surface sketch

Using `tdm` as the working CLI binary name:

```text
tdm init
tdm list
tdm show <id>
tdm add --title ... --state todo
tdm move <id> --state review
tdm complete <id> --summary ...
tdm reopen <id> --state active
tdm log list|show|search
tdm accord ready|claim|deliver|accept|rework|block
tdm review request|accept|changes
tdm rules list|add|edit|delete
tdm tui
tdm import brainfile
tdm lint --fix
```

## Brainfile import compatibility

A migration/import command should be able to convert Brainfile v2 boards:

```text
.brainfile/brainfile.md → .tandem/tandem.md
.brainfile/board/*.md  → .tandem/board/*.md
.brainfile/logs/*.md   → .tandem/logs/*.md
logs/ledger.jsonl      → events.jsonl, if compatible
contract               → accord
column                 → state
```

The importer should preserve original files by default and write a migration report.

## Open questions

- Confirm Tandem as the final project/protocol name and `tdm` as the CLI binary name.
- Confirm `accord` as the final replacement for Brainfile's `contract` object.
- Should default states be `todo/active/review` or `backlog/todo/active/review`?
- Should completion require accepted review by default?
- Should completion require an accepted accord when an accord exists?
- Should IDs be ULID-based, slug-based, or sequential with branch-safe allocation?
- How much schema strictness should v0 enforce?
- Should the event ledger be named `events.jsonl`, `ledger.jsonl`, or `history.jsonl`?
- Should logs be reconstructed from archived Markdown, events, or a merged view?
