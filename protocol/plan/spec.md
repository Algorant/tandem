# Tandem Protocol Spec Draft

Status: v0 draft accepted for implementation
Date: 2026-06-26  
Working name: Tandem

This document sketches the Tandem protocol: a Brainfile-inspired file format that keeps Brainfile's useful shape, adapts the language for Tandem, and folds in local v3 improvements around review, complete/archive, and first-class logs.

The protocol is the spec/source of truth for Tandem's local-first project coordination files. It is not an implementation package or crate layout.

Acceptance stamp: the v0 draft is complete enough to implement `tandem init`, `tandem list`, `tandem show`, and subsequent v0 mutations. Future protocol edits should come from implementation feedback or explicit product decisions, not abstract planning churn.

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
- Default active states are `todo`, `in-progress`, and `validation`; existing `state: review` documents are tolerated as a legacy alias during the transition.
- New task items use `type: task`; default allocation uses flat sequential IDs such as `task-1`.
- First-class document types are `task` and `decision`.
- Custom document types are allowed in config only; v0 has no type-management CLI.
- A first-class subtask is a normal `type: task` document whose `parentId` points to another task. It keeps its own workflow, accord, review, ownership, and completion behavior; no subtask document type or separate relationship field is needed.
- Parent-derived hierarchical IDs such as `task-100-1` are allowed and recommended when they improve local readability, but they are optional. `parentId`, not ID shape, defines the hierarchy.
- Inline `subtasks:` checklist objects are legacy and deprecated for new work. Tools must continue to read, validate, preserve, and operate on existing entries, but new independently trackable work should be a child task document.
- Epics are a convention on task documents: use `type: task` plus `kind: epic` for broad outcome grouping, link children through the same general `parentId` hierarchy, and use `references` for loose related documents. A parent need not be an epic, and no separate epic document type, command family, or lifecycle is part of v0.
- The work-agreement object is `accord`.
- Accord statuses are `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, and `blocked`.
- Rules are structured objects with stable IDs: `{ id, rule, source? }`.
- `parentId`, `blockers`, and `references` may point to any Tandem document by ID. A task is a first-class subtask specifically when its `parentId` target is another task.
- Completion warns about missing accepted review or accepted accord, but allows completion in v0.
- Protocol version for the first v0 draft is `0.1.0`.
- Events live in per-actor append logs under `.tandem/events/<actor_id>.jsonl`; legacy `.tandem/events.jsonl` remains a readable transition source, but new writes should not append to it by default.
- Event payloads are minimal audit records in v0: require `ts`, `event`, `id`, `summary`, `actor`, and `seq`; event identity is `<actor>:<seq>`; defer typed per-event payload schemas.
- Completed logs are archived Markdown documents in `.tandem/logs/`; those documents are the source of truth for completed history. Events enrich timeline and audit views.
- Validation/lint is built-in structural validation only in v0.
- Schemas and fixtures are not part of v0.
- Validation severity is strict for structure and core references: invalid/missing required structure and unresolved `parentId`/`blockers` are errors; unresolved related `references`/rule sources and completion-policy issues are warnings.
- Decision documents do not need a workflow lifecycle `state` field in v0. They may carry an ADR-style `status` such as `proposed` or `accepted`; this status is decision metadata, not task workflow state.

## Naming model

- Product/protocol: **Tandem**
- Repository: `tandem`
- Protocol data directory: `.tandem/`
- Protocol config file: `.tandem/tandem.md`
- CLI binary: `tandem`
- CLI/TUI work directory in this monorepo: `tandem/`
- User-facing CLI: `tandem`; reserve `td` for future/internal tool prefixes
- Work-agreement object replacing Brainfile's `contract` concept: `accord`

`tandem` is intentionally short enough for daily terminal use while still reading as Tandem. `td` is reserved for future/internal tool prefixes unless explicitly revisited.

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
├── events/            # per-actor append-only lifecycle events
│   ├── actor_01HV9ABC.jsonl
│   └── actor_01HV9XYZ.jsonl
└── events.jsonl       # legacy global event log; readable during transition
```

Tool-specific sidecars may exist under `.tandem/`. The current TUI reads optional `.tandem/config.toml` for display preferences such as Board tag badge opt-ins; this is not the protocol workspace config and must not replace `.tandem/tandem.md`.

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
todo → in-progress → validation → complete/archive → logs
```

Completed work belongs in `.tandem/logs/` and is enriched by aggregated per-actor event logs under `.tandem/events/` plus any legacy `.tandem/events.jsonl`.

Projects may define additional active states, but `todo`, `in-progress`, and `validation` are the v0 defaults. Existing `state: review` files are legacy-compatible reads; new writes should prefer `validation`. A persistent completed board state is not part of the default workflow.

### 2. Separate human workflow state from agent accord state

Brainfile's contract status and task column can drift or overlap conceptually. Tandem makes the layers explicit:

```yaml
state: validation          # human/project workflow state
accord:
  status: delivered        # agent/human agreement state
review:
  status: pending          # validation/review state
```

This allows a task to be in workflow `validation` while the accord is `delivered`, `accepted`, or `rework`.

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

### 5. Task IDs support flat and optional hierarchical sequences

New v0 tasks default to flat sequential IDs:

```yaml
id: task-1
type: task
```

First-class child tasks may instead use a parent-derived hierarchical ID when that makes the relationship easier to scan:

```yaml
id: task-100-1
type: task
parentId: task-100
```

Hierarchical IDs are optional and may add further positive-integer segments for deeper nesting. `parentId` is always the canonical relationship; tools must not infer or require hierarchy from an ID alone. Default automatic allocation remains the flat global `task-N` sequence unless the caller explicitly supplies a hierarchical ID through capable tooling or manual authoring.

Suggested filenames:

```text
task-1-implement-theme-system.md
task-100-1-add-child-validation.md
```

The ID is canonical. A readable filename suffix may change without changing the document identity.

Decision documents use the same pattern with their own prefix:

```yaml
id: decision-1
type: decision
```

Legacy inline checklist subtasks may still appear as:

```yaml
subtasks:
  - id: task-1-1
    title: Define theme behavior
    completed: false
```

These entries are not task documents and do not gain workflow, ownership, accord, review, blocker, or completion semantics. The field is deprecated for new work; tools must read and preserve existing entries, while new work should use normal child task documents linked with `parentId`.

Custom document types may define their own `idPrefix` in workspace config, but v0 does not include type-management commands.

## Discovery

Tools should discover a Tandem workspace in this order:

1. `.tandem/tandem.md`
2. `tandem.md` in the repository root, for simple/single-file compatibility within Tandem itself
3. no legacy Brainfile discovery paths in v0

Discovery should stop at repository boundaries unless explicitly told to search parent directories.

## V0 field reference

This section is the implementation-facing reference for v0 frontmatter and JSONL records. Unknown fields are allowed and must be preserved by compliant tools unless a user explicitly formats or removes them.

Severity notes:

- **error** means structural validation should fail and mutating commands should refuse to proceed unless an explicit recovery/force flow exists.
- **warning** means tools should report the issue but may proceed.
- Missing or malformed required structure is an error.
- Unresolved core references, `parentId` and `blockers`, are errors.
- Unresolved related references, rule sources, and completion-policy issues are warnings.

### Workspace config fields

Workspace config lives in `.tandem/tandem.md` frontmatter.

| Field | Required | Severity | Notes |
| --- | --- | --- | --- |
| `protocolVersion` | yes | error | Must be `0.1.0` for this v0 draft. |
| `title` | yes | error | Human-readable workspace title. |
| `states` | yes | error | Array of workflow states. `tandem init` writes `todo`, `in-progress`, `validation`. Duplicate IDs are errors. Missing defaults are warnings if no active task uses them. Existing `review` is a legacy alias for `validation` reads. |
| `completion` | no | warning | Completion policy hints. V0 warns but does not block when review/accord acceptance is missing. |
| `types` | no | error if malformed | Defines first-class and custom document type metadata. Custom types are config-only in v0. |
| `rules` | no | error if malformed | Rule groups `always`, `never`, `prefer`, `context`; each entry is a rule object. |
| `agent` | no | warning if malformed | Agent-facing instructions. Unknown nested fields should be preserved. |
| `theme` | no | none | Protocol stores the value but CLI/TUI owns interpretation. |
| `views` | no | none | Optional saved views/filters; preserved if present. |

### Task document fields

Task documents live in `.tandem/board/` while active and `.tandem/logs/` after completion/archive.

| Field | Required | Severity | Notes |
| --- | --- | --- | --- |
| `id` | yes | error | Canonical ID. Task IDs use one or more positive-integer segments, e.g. flat `task-1` or optional hierarchical `task-100-1`. IDs must be unique across board and logs; ID shape does not establish hierarchy. |
| `type` | yes | error | Must be `task` for v0 task documents. |
| `kind` | no | error if unsupported | Optional task sub-kind/convention classifier. Omitted means a normal task; v0 defines `epic` for lightweight grouping/planning tasks without changing task identity, ID allocation, workflow, accord, review, or completion behavior. |
| `title` | yes | error | Display title. |
| `state` | yes in board | error | Must match a configured workspace state. Omitted in logs is allowed. |
| `priority` | no | warning if unrecognized | Suggested values: `low`, `medium`, `high`, `critical`; projects may extend. |
| `effort` | no | warning if unrecognized | Suggested values: `trivial`, `small`, `medium`, `large`, `xlarge`; projects may extend. |
| `tags` | no | error if malformed | Array of strings. |
| `assignee` | no | none | Human or agent responsible. |
| `parentId` | no | error if unresolved | Core reference to any Tandem document ID. When a task points to another task, this field makes it a first-class child/subtask; no other relationship field or task type is required. |
| `blockers` | no | error if unresolved | Core references to Tandem document IDs blocking this task. |
| `references` | no | warning if unresolved | Related Tandem document IDs. |
| `relatedFiles` | no | warning if path malformed | Project paths relevant to the task; paths do not have to exist. |
| `accord` | no | error if malformed | Work-agreement object. If present, `accord.status` is required. |
| `review` | no | error if malformed | Review object. If present, `review.status` is required. |
| `subtasks` | no | error if malformed | Legacy inline checklist array of `{ id, title, completed }`; deprecated for new work. Existing entries retain parent-based sequential values such as `task-1-1` and must remain readable/preservable. |
| `createdAt` | no | warning if malformed | Timestamp for creation. |
| `updatedAt` | no | warning if malformed | Timestamp for last mutation. |
| `completedAt` | logs only | error in logs if missing | Timestamp for completion/archive. Active tasks should not normally carry it. |
| `completion` | logs only | error in logs if missing | Completion metadata; see below. |

`kind` does not change task identity: epic tasks still use `type: task`, the normal task ID namespace and shapes, normal board states, and normal completion/archive behavior. Child work links to an epic through `parentId`; looser association remains `references`.

### Decision document fields

Decision documents are first-class v0 documents. They live in `.tandem/board/` and capture durable project, product, or architecture choices. A Tandem decision is the ADR-compatible record type; do not introduce a separate `adr` document type or workflow state for architecture decisions. Decision documents do not need a workflow lifecycle `state` field in v0. ADR-style `status` is decision metadata and must remain distinct from task workflow state.

| Field | Required | Severity | Notes |
| --- | --- | --- | --- |
| `id` | yes | error | Canonical ID. New decision IDs are sequential, e.g. `decision-1`. |
| `type` | yes | error | Must be `decision`. |
| `title` | yes | error | Display title. |
| `status` | no | warning if unrecognized | ADR-style decision status. Suggested values: `proposed`, `accepted`, `rejected`, `deprecated`, `superseded`. This is decision metadata, not task workflow `state`. New CLI-created decisions default to `proposed`. |
| `date` | no | warning if malformed | ADR decision date, usually `YYYY-MM-DD`. New CLI-created decisions default to the current UTC date. |
| `deciders` | no | warning if malformed | Array of humans/agents responsible for the decision. |
| `context` | no | none | Brief ADR context summary; longer context may live in the Markdown body. |
| `consequences` | no | warning if malformed | Array of consequence summaries. Longer discussion may live in the Markdown body. |
| `alternatives` | no | warning if malformed | Array of considered alternatives. |
| `supersedes` | no | warning if unresolved | Decision IDs this decision supersedes. Mirror IDs in `references` for v0 tool-visible relationships. |
| `supersededBy` | no | warning if unresolved | Decision IDs that supersede this decision. Mirror IDs in `references` for v0 tool-visible relationships. |
| `references` | no | warning if unresolved | Related Tandem document IDs. Include superseded/superseding decision IDs here when they should be visible to current CLI/TUI search and relationship views. |
| `tags` | no | error if malformed | Array of strings. Use tags such as `adr`, `architecture`, `product`, or area names for filtering. |
| `createdAt` | no | warning if malformed | Timestamp for creation. |
| `updatedAt` | no | warning if malformed | Timestamp for last mutation. |

### Accord object fields

`accord` is optional on a task. If present, it must be an object with a canonical status.

| Field | Required | Severity | Notes |
| --- | --- | --- | --- |
| `status` | yes | error | One of `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`. |
| `assignee` | no | none | Human or agent responsible for the accord. |
| `claimedAt` | no | warning if malformed | Timestamp when claimed. |
| `deliveredAt` | no | warning if malformed | Timestamp when delivered. |
| `summary` | no | none | Current delivery or agreement summary. |
| `deliverables` | no | warning if malformed | Array of deliverable objects. Suggested fields: `type`, `path`, `description`, `required`. |
| `validation` | no | warning if malformed | Validation expectations such as `commands`; v0 lint does not execute them. |
| `evidence` | no | warning if malformed | Recorded evidence such as command results, file paths, or notes. |
| `constraints` | no | warning if malformed | Scope constraints. |
| `outOfScope` | no | warning if malformed | Explicit exclusions. |

### Review object fields

`review` is optional on a task. If present, it must be an object with a canonical review state.

| Field | Required | Severity | Notes |
| --- | --- | --- | --- |
| `status` | yes | error | One of `not-ready`, `pending`, `accepted`, `changes-requested`, `rejected`. |
| `reviewer` | no | none | Human/agent reviewer identifier. |
| `requestedAt` | no | warning if malformed | Timestamp when review was requested. |
| `decidedAt` | no | warning if malformed | Timestamp when accepted/rejected/changes requested. |
| `notes` | no | warning if malformed | Array of review notes or note objects. |

### Completion metadata fields

Completed task documents in `.tandem/logs/` should include `completedAt` and `completion` metadata. Missing required log metadata is an error because logs are the completed-work source of truth.

| Field | Required | Severity | Notes |
| --- | --- | --- | --- |
| `completedAt` | yes in logs | error | Completion/archive timestamp. |
| `completion.summary` | yes in logs | error | Human-readable completion summary. |
| `completion.filesChanged` | no | warning if malformed | Array of project paths changed. |
| `completion.validation` | no | warning if malformed | Recorded validation result summary; v0 lint does not execute commands. |
| `completion.reviewer` | no | none | Reviewer or completer identifier. |
| `completion.notes` | no | warning if malformed | Additional completion notes. |

### Log document expectations

Archived Markdown documents in `.tandem/logs/` are the completed-work source of truth. Events may enrich timelines, but a log document must remain understandable without replaying any event log.

A valid completed task log should contain:

- original task identity fields: `id`, `type: task`, `title`
- `completedAt`
- `completion.summary`
- any retained `accord`, `review`, legacy inline `subtasks`, `references`, `blockers`, and `relatedFiles`
- the original or final Markdown body

### Rule object fields

Rules are structured objects inside one of the workspace config groups: `always`, `never`, `prefer`, or `context`.

| Field | Required | Severity | Notes |
| --- | --- | --- | --- |
| `id` | yes | error | Stable rule ID such as `rule-1`. |
| `rule` | yes | error | Human-readable rule text. |
| `source` | no | warning if unresolved | Tandem document ID explaining where the rule came from. |

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
  - id: validation
    title: Validation
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

The v0 spec uses built-in structural validation. Remote schema URLs, fixture directories, and schema-management commands are deferred out of v0.

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
---

## Description

Build a user-configurable theme layer for the Ratatui TUI.

## Notes

Freeform notes stay in Markdown and should not be destroyed by tools.
```

### Task fields

| Field | Required | Purpose |
| --- | --- | --- |
| `id` | yes | Stable canonical identifier such as flat `task-1` or optional hierarchical `task-100-1`; the ID does not define parentage. |
| `type` | no | Defaults to `task` for new task documents. |
| `kind` | no | Optional task sub-kind/convention classifier. Omit for normal tasks; use `epic` for lightweight planning/grouping tasks while preserving `type: task` and normal task semantics. |
| `title` | yes | Display title. |
| `state` | yes for active tasks | Human workflow state. Defaults are `todo`, `in-progress`, and `validation`; `review` is read as a legacy alias for validation. |
| `priority` | no | `low`, `medium`, `high`, `critical`, or project-defined. |
| `effort` | no | `trivial`, `small`, `medium`, `large`, `xlarge`, or project-defined. |
| `tags` | no | Filtering/grouping. |
| `assignee` | no | Human or agent currently responsible. |
| `parentId` | no | Parent Tandem document ID. A task pointing to another task is a first-class child/subtask; the field may otherwise point to any document type. |
| `references` | no | Related Tandem document IDs. May point to any document type. |
| `relatedFiles` | no | Project paths relevant to the task. |
| `blockers` | no | Tandem document IDs blocking this item. May point to any document type. |
| `accord` | no | Agent/human work agreement. |
| `review` | no | Review and validation state. |
| `subtasks` | no | Legacy inline checklist items with parent-based sequential IDs. Read and preserve existing values; deprecated for new work in favor of child task documents. |
| `createdAt` | no | Creation timestamp. |
| `updatedAt` | no | Last mutation timestamp. |
| `completedAt` | logs only | Completion timestamp. |

## Parent/child task and subtask convention

A first-class subtask is an ordinary task document linked to another task with `parentId`. It has the same schema and lifecycle as every other task, including its own `state`, assignee, blockers, accord, review, and completion record. Tandem does not need a `subtask` document type, a `subtaskOf` field, or another relationship mechanism.

Example parent task:

```markdown
---
id: task-100
type: task
title: Improve relationship modeling
state: in-progress
---
```

Example first-class child task using an optional hierarchical ID:

```markdown
---
id: task-100-1
type: task
title: Document child task validation
state: todo
parentId: task-100
---
```

The same child could use a flat ID such as `task-137`; `parentId: task-100` alone establishes the relationship. Parent-derived IDs are allowed and recommended when useful for local readability, but are never required, and tools must not infer parentage from them. An ordinary task may parent children without `kind: epic`, and nesting may continue by linking a child task to another child task.

Inline `subtasks:` objects remain a legacy checklist representation, not first-class task documents. Compliant tools should continue to read, validate, preserve, and toggle existing entries without silently converting them. New work that needs tracking should be authored as a child task document; migration of an old inline item should be explicit so identity and history are not fabricated.

## Epic convention

Epics are normal tasks with `kind: epic`: they remain board-visible, use normal workflow states, allocate normal task IDs, and complete/archive into logs like any other task. Use an epic when a task mainly groups a larger outcome or milestone. `kind: epic` describes the parent's planning role; it does not create the parent/child relationship, and general task hierarchies do not require an epic.

Example epic task:

```markdown
---
id: task-10
type: task
kind: epic
title: Ship documentation IA refresh
state: in-progress
priority: high
references:
  - decision-3
relatedFiles:
  - docs/index.md
---

## Outcome

Coordinate the docs information architecture refresh across smaller child tasks.
```

Example child task:

```markdown
---
id: task-11
type: task
title: Rewrite Concepts page
state: todo
parentId: task-10
references:
  - decision-3
  - task-7
relatedFiles:
  - docs/concepts/index.md
---

## Description

Update the public concepts page as one deliverable under the docs IA epic.
```

Guidance:

- Keep epic IDs in the normal task namespace (usually flat `task-N`, with optional hierarchical task IDs still allowed), not a separate `epic-N` namespace.
- Use `parentId` for the strict parent/child hierarchy. The target may be an active board document or a completed log document, and unresolved `parentId` remains a validation error.
- Use `references` for loose related context such as decisions, sibling tasks, or completed logs. References are intentionally not hierarchy and unresolved references remain warnings.
- Use normal child task documents when work needs its own owner, accord, review, blockers, validation, or completion history. Existing inline `subtasks` remain legacy checklist data and are deprecated for new work.
- Complete/archive an epic with the normal task completion flow only when its children are completed, intentionally canceled/superseded, or the human/project owner decides the epic is done. Completion moves the epic task to `.tandem/logs/`; it does not create a persistent `done` state or special epic archive.
- Do not model epics as `type: decision`, ADRs, custom protocol folders, or separate lifecycle states. Decisions remain for durable choices; epics remain task coordination records.

## Decision document

Decision documents are first-class v0 documents. They capture durable project, product, or architecture decisions and may be referenced by tasks, blockers, rules, and other decisions. They are intentionally ADR-compatible without introducing a separate ADR type: use `type: decision`, optional ADR-style metadata, and a Markdown body shaped like an ADR.

Example:

```markdown
---
id: decision-1
type: decision
title: Use accord vocabulary for work agreements
status: accepted
date: 2026-06-26
deciders:
  - Algorant
  - Pi
context: Brainfile's contract term feels legalistic for Tandem's collaborative workflow.
alternatives:
  - Keep contract vocabulary
  - Use assignment only
consequences:
  - Work-agreement metadata uses `accord` consistently.
tags: [adr, protocol]
createdAt: 2026-06-26T12:00:00Z
updatedAt: 2026-06-26T12:10:00Z
references:
  - task-1
supersedes: []
supersededBy: []
---

## Status

Accepted.

## Context

Tandem needs a name for the explicit human/agent work-agreement object.

## Decision

Use `accord` for the collaborative work-agreement object.

## Context

Brainfile's `contract` term is technically clear but feels legalistic and one-sided for Tandem.

## Alternatives considered

- Keep `contract` vocabulary.
- Use assignment metadata without a named work-agreement object.

## Consequences

The term is less legalistic than Brainfile's contract terminology and better matches Tandem's collaborative tone. CLI, TUI, protocol, and agent guidance should use `accord` consistently.

## Supersession

- Supersedes: none
- Superseded by: none
```

Recommended ADR-compatible body template:

```markdown
## Status

Accepted, proposed, superseded, deprecated, or rejected. This mirrors optional `status:` metadata and is not task workflow `state`.

## Context

What forces, constraints, alternatives, or prior decisions made this choice necessary?

## Decision

What has been decided?

## Consequences

What becomes easier, harder, required, or intentionally deferred?

## Supersession

- Supersedes: decision-N or none
- Superseded by: decision-M or none

## References

- task-N, decision-N, log-N, code/docs paths, or external links as needed
```

Required v0 decision fields are `id`, `type: decision`, and `title`. Decision documents do not need a workflow lifecycle `state` in v0. Their durable decision content lives in frontmatter metadata plus the Markdown body. `status`, `date`, `deciders`, `context`, `consequences`, `alternatives`, `supersedes`, and `supersededBy` are the recommended ADR-compatible fields; `status` is not task workflow state. Use `references` as the canonical tool-visible link graph, including supersession relationships when they need to appear in CLI/TUI search or relationship displays.

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
| `delivered` | `validation` |
| `accepted` | `validation` until completion/archive |
| `rework` | `in-progress` or `validation`, depending on project preference |
| `failed` | any state plus failure indicator |
| `blocked` | any state plus blocked indicator |

The protocol should preserve the distinction between accord state and human workflow state. TUI/CLI tools should make misalignment visible rather than silently hiding it.

## Review model

```yaml
review:
  status: pending     # not-ready | pending | accepted | changes-requested | rejected
  reviewer: Algorant
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
3. Appends a completion event to the current actor's per-actor event log under `.tandem/events/<actor_id>.jsonl`.
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
  reviewer: Algorant
accord:
  status: accepted
```

Archived Markdown documents in `.tandem/logs/` are the source of truth for completed history. Events enrich timeline, audit, and search views, but tools should not need events to reconstruct the current board or completed-log corpus.

## Events

New Tandem event storage uses per-actor append logs:

```text
.tandem/
├── events/
│   ├── actor_01HV9ABC.jsonl
│   └── actor_01HV9XYZ.jsonl
└── events.jsonl       # legacy global log; read-only transition source for new tools
```

Each `actor_id` is the canonical writer identity and event-log ownership key. It must be unique enough for file ownership and safe as a filename segment. Tools may reuse an actor ID from user-level or workspace-local actor configuration when one exists. If no existing actor ID can be found, Tandem may generate a new opaque ID and continue rather than blocking; the primary goal is disentangling Git appends, not perfect machine or human recognition. Tools should avoid sharing one actor ID among independent concurrent writers.

Each writer must append only to `.tandem/events/<actor_id>.jsonl` for its own canonical `actor_id`. New writes should not append to the legacy `.tandem/events.jsonl` by default. Readers aggregate all readable `.tandem/events/*.jsonl` files plus `.tandem/events.jsonl` when it exists, so workspaces can transition without losing audit history.

V0 event records use a minimal audit envelope. Required fields:

| Field | Required | Purpose |
| --- | --- | --- |
| `ts` | yes | Event timestamp. |
| `event` | yes | Event name from the catalog below. |
| `id` | yes | Primary subject ID. Usually a Tandem document ID; rule events may use a rule ID. |
| `summary` | yes | Human-readable audit summary. |
| `actor` | yes | Canonical actor ID that owns the event log file. Must match `<actor_id>` in `.tandem/events/<actor_id>.jsonl` for per-actor logs. |
| `seq` | yes | Per-actor monotonically increasing sequence number. Event identity is `<actor>:<seq>`. |
| `actorName` | no | Optional cosmetic display name populated from user/project configuration when available. Never canonical identity and never file ownership. |
| `details` | no | Freeform JSON object for extra context. |

Writers assign `seq` within their own actor log; `seq` is not globally ordered across actors. Gaps are acceptable after repair or failed writes, but duplicate `<actor>:<seq>` identities are invalid within the aggregated event set. Legacy records read from `.tandem/events.jsonl` may lack `actor` and `seq`; readers should tolerate them as legacy audit records and may display a synthetic source such as `legacy:<line>` without treating it as a canonical event identity.

V0 intentionally does not define typed per-event payload schemas. Consumers must tolerate unknown optional fields.

Example per-actor events:

```jsonl
{"ts":"2026-06-26T12:00:00Z","event":"task.created","id":"task-1","summary":"Created task: Implement Ratatui theme system","actor":"actor_01HV9ABC","seq":1,"actorName":"Algorant"}
{"ts":"2026-06-26T12:05:00Z","event":"accord.claimed","id":"task-1","summary":"pi claimed the accord","actor":"actor_01HV9XYZ","seq":1,"actorName":"pi"}
{"ts":"2026-06-26T13:30:00Z","event":"accord.delivered","id":"task-1","summary":"Initial implementation ready","actor":"actor_01HV9XYZ","seq":2}
{"ts":"2026-06-26T15:00:00Z","event":"task.completed","id":"task-1","summary":"Completed and archived to logs","actor":"actor_01HV9ABC","seq":2,"actorName":"Algorant"}
```

Per-actor logs remove the shared Git file-level append hotspot. They do not eliminate semantic conflicts: two actors can still make conflicting task, accord, review, rule, or decision changes. Readers should preserve all actor events and may surface or detect contradictory histories; resolution UX and resolution-specific events are out of scope for the minimum per-actor event-log change.

Events should never be required to reconstruct the current board. They provide audit/history and power richer logs.

### Event name catalog

Task events:

- `task.created`
- `task.updated`
- `task.moved`
- `task.completed`

Decision events:

- `decision.created`
- `decision.updated`

Accord events:

- `accord.ready`
- `accord.claimed`
- `accord.delivered`
- `accord.accepted`
- `accord.rework`
- `accord.failed`
- `accord.blocked`

Review events:

- `review.requested`
- `review.accepted`
- `review.changes_requested`
- `review.rejected`
- `review.note_added`

Completion/archive events:

- `task.completed` — completion/archive mutation from `.tandem/board/` to `.tandem/logs/`.

Restore/reopen events, post-v0 names reserved:

- `task.restored`
- `task.reopened`

Rule events:

- `rules.added`
- `rules.updated`
- `rules.deleted`

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

Rules are for humans and agents. Agents should read them before starting tasks. `source` is optional and may point to any Tandem document ID.

## Document types

First-class v0 types:

- `task` — normal task/work item. Completable by default.
- `decision` — durable project, product, or architecture decision. Not completable by default.

Epics use the `task` document type with the optional `kind: epic` convention. They are not a first-class v0 type and should not require custom type configuration.

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

## Validation diagnostics

V0 validation/lint is built-in structural validation only. It does not execute project validation commands, enforce remote schemas, run hooks, perform auth checks, generate fixture data, or manage custom type definitions.

Diagnostics should include at least: severity, code, path, document ID when known, field path when known, and a human-readable message.

### Error categories

Errors fail validation and should block normal mutations.

| Code | Category | Example |
| --- | --- | --- |
| `E001` | workspace config unreadable | `.tandem/tandem.md` is missing or frontmatter cannot be parsed. |
| `E002` | workspace required field missing | Missing `protocolVersion`, `title`, or `states`. |
| `E003` | unsupported protocol version | `protocolVersion` is not `0.1.0`. |
| `E010` | document frontmatter unreadable | A Markdown file in `board/` or `logs/` has invalid frontmatter. |
| `E011` | document required field missing | A task is missing `id`, `type`, `title`, or active `state`. |
| `E012` | duplicate document ID | The same ID appears in more than one board/log document. |
| `E013` | invalid document ID shape | A v0 task ID is not `task-` followed by one or more positive-integer segments, or a decision ID is not `decision-N`. |
| `E020` | unknown document type | `type` is neither `task`, `decision`, nor a configured custom type. |
| `E021` | unknown active state | A task has `state: blocked` but `blocked` is not in workspace `states`. |
| `E030` | invalid accord object | `accord.status` is missing or not one of the canonical v0 values. |
| `E031` | invalid review object | `review.status` is missing or not one of the documented review values. |
| `E040` | invalid rule object | A rule entry is not an object or lacks `id`/`rule`. |
| `E050` | invalid legacy inline subtask object | A retained inline checklist entry lacks `id`, `title`, or `completed`, or its ID does not match the parent-based sequential pattern. |
| `E060` | unresolved core parent | `parentId` points to no Tandem document. |
| `E061` | unresolved core blocker | A `blockers` entry points to no Tandem document. |
| `E070` | invalid completed log | A log task lacks `completedAt` or `completion.summary`. |

### Warning categories

Warnings should be shown, but tools may proceed.

| Code | Category | Example |
| --- | --- | --- |
| `W010` | unresolved related reference | A `references` entry points to no known Tandem document. |
| `W011` | unresolved rule source | A rule `source` points to no known Tandem document. |
| `W020` | completion review policy | A task is being completed without `review.status: accepted`. |
| `W021` | completion accord policy | A task with an accord is being completed without `accord.status: accepted`. |
| `W030` | missing default state | Workspace `states` omits one of the default states and no active task currently needs it. |
| `W040` | malformed optional metadata | Optional timestamp, priority, effort, notes, evidence, or validation metadata is malformed but recoverable. |
| `W050` | non-canonical preserved field | A field is unknown to v0 but can be preserved safely. |

### Reference validation

Reference checks should build an ID index from both `.tandem/board/` and `.tandem/logs/`.

- `parentId` and `blockers` are core references. Missing targets are errors.
- `references` and rule `source` are related references. Missing targets are warnings.
- References may point to any Tandem document type, including completed log documents.

### Completion-policy validation

Completion-policy findings are warnings in v0. `tandem complete` should warn when review or accord acceptance is missing, then allow completion unless structural errors are present.

## Mutation semantics

This section defines the v0 protocol-level effects of mutating operations. It intentionally does not define minimal-diff write mechanics; compliant tools still preserve unknown fields and Markdown bodies as described below.

All mutations should:

- discover and read `.tandem/tandem.md`
- validate required structure before writing
- update `updatedAt` on the changed document when applicable
- append one minimal audit event to the current actor's `.tandem/events/<actor_id>.jsonl` unless the operation fails before mutation
- preserve unrelated files

Event records use the v0 audit envelope: `ts`, `event`, `id`, `summary`, `actor`, and `seq`, with optional `actorName` and `details`.

### Add task

| Aspect | Semantics |
| --- | --- |
| Required inputs | `title`; optional `state`, `kind`, body/description, priority, effort, tags, assignee, `parentId`, `blockers`, `references`, `relatedFiles`, deprecated inline subtasks for compatibility, accord, review. New callers should create child task documents instead of inline entries. |
| Files read | `.tandem/tandem.md`, `.tandem/board/*.md`, `.tandem/logs/*.md` for ID allocation and reference validation. |
| Files written | New `.tandem/board/task-N*.md`; append current actor event log under `.tandem/events/<actor_id>.jsonl`. |
| Validation/errors/warnings | Error if workspace is invalid, generated ID would duplicate, requested `state` is not configured, requested `kind` is unsupported, `parentId`/`blockers` are unresolved, or nested accord/review/subtasks are malformed. Warn for unresolved `references`, unresolved rule sources, malformed optional metadata, or omitted default state. |
| Event | `task.created`. |
| Resulting state | New task document in `.tandem/board/` with `id: task-N`, `type: task`, optional `kind`, `title`, `state` defaulting to `todo`, `createdAt`, and `updatedAt`. |

Default task ID allocation chooses the next available flat `task-N` value after scanning existing task IDs in both board and logs. Parent-derived hierarchical IDs may be supplied explicitly through capable tooling or manual authoring, but `parentId` remains the canonical relationship. Human-readable filename suffixes are optional and non-canonical.

### Add decision

| Aspect | Semantics |
| --- | --- |
| Required inputs | `title`; optional body/decision text, ADR `status`, `date`, repeated `deciders`, `context`, repeated `consequences`, repeated `alternatives`, repeated `supersedes`, repeated `supersededBy`, `references`, tags, and `createdAt`/`updatedAt` override only for trusted bulk-authoring tooling outside normal v0 commands. |
| Files read | `.tandem/tandem.md`, `.tandem/board/*.md`, `.tandem/logs/*.md` for ID allocation and related reference/supersession validation. |
| Files written | New `.tandem/board/decision-N*.md`; append current actor event log under `.tandem/events/<actor_id>.jsonl`. |
| Validation/errors/warnings | Error if workspace is invalid, generated ID would duplicate, or a requested decision `status` is outside the recommended ADR values. Warn for unresolved `references`, unresolved/mis-typed `supersedes`/`supersededBy`, or malformed optional timestamps/arrays. No decision workflow `state` field is required or written in v0. |
| Event | `decision.created`. |
| Resulting state | New decision document in `.tandem/board/` with `id: decision-N`, `type: decision`, `title`, ADR metadata (`status` defaulting to `proposed` and `date` defaulting to the current UTC date for CLI-created decisions), `createdAt`, `updatedAt`, and Markdown body. |

Decision ID allocation chooses the next available positive integer after scanning existing decision IDs in both board and logs.

### Move state

| Aspect | Semantics |
| --- | --- |
| Required inputs | Task ID and target `state`. |
| Files read | `.tandem/tandem.md`, target task document in `.tandem/board/`, and enough document index data to validate core references. |
| Files written | Target task document; append current actor event log under `.tandem/events/<actor_id>.jsonl`. |
| Validation/errors/warnings | Error if the ID is missing, resolves to a non-task document, resolves only in `.tandem/logs/`, target `state` is not configured, target task has unresolved `parentId`/`blockers`, or task structure is invalid. Warn for unresolved related `references` or state/accord visual misalignment. |
| Event | `task.moved`. |
| Resulting state | Existing active task remains in `.tandem/board/` with updated `state` and `updatedAt`. |

Moving state preserves review metadata and usually preserves accord status, but tools may apply conservative paired synchronization for common non-destructive transitions. In v0, moving a task from `todo` to `in-progress` may claim an existing `accord.status: ready`; other ambiguous state-to-accord changes should warn or require explicit accord commands. The protocol keeps workflow state, accord status, and review metadata separate.

### Update accord

| Aspect | Semantics |
| --- | --- |
| Required inputs | Task ID and requested accord status/action. Optional assignee, summary, deliverables, validation expectations, evidence, constraints, and out-of-scope entries. |
| Files read | `.tandem/tandem.md`, target task document in `.tandem/board/`, and document index for core reference validation. |
| Files written | Target task document; append current actor event log under `.tandem/events/<actor_id>.jsonl`. |
| Validation/errors/warnings | Error if the ID is missing, resolves to a log document or non-task document, requested status is not canonical, accord object would be malformed, or existing core references are unresolved. Warn for unresolved related `references`, completion-policy issues when relevant, or state/accord visual misalignment. |
| Event | Status-specific event: `accord.ready`, `accord.claimed`, `accord.delivered`, `accord.accepted`, `accord.rework`, `accord.failed`, or `accord.blocked`. |
| Resulting state | Task stays in `.tandem/board/`; `accord.status` and related accord fields are updated. Tools may also apply conservative state synchronization for compatible states: `claimed` moves `todo` to `in-progress`; `delivered`/`accepted` move `todo`, `in-progress`, or legacy `review` to `validation`; `rework` may move `validation`/legacy `review` back to `in-progress`. `blocked` and `failed` remain cross-cutting signals and do not automatically move workflow state. |

Suggested visual alignment remains: `ready` with `todo`, `claimed` with `in-progress`, `delivered`/`accepted` with `validation`. Existing `review` states should be displayed as legacy validation. Misalignment is allowed but should be visible in tools as a warning rather than silently hidden.

### Request review

| Aspect | Semantics |
| --- | --- |
| Required inputs | Task ID. Optional reviewer and notes. |
| Files read | `.tandem/tandem.md`, target task document in `.tandem/board/`, and document index for core reference validation. |
| Files written | Target task document; append current actor event log under `.tandem/events/<actor_id>.jsonl`. |
| Validation/errors/warnings | Error if the ID is missing, resolves to a log document or non-task document, review object would be malformed, or existing core references are unresolved. Warn for unresolved related `references` or state/review mismatch if the task is not in `validation` (or legacy `review`). |
| Event | `review.requested`. |
| Resulting state | Task stays in `.tandem/board/`; `review.status` becomes `pending`, `requestedAt` is set, optional reviewer/notes are recorded. Protocol does not automatically move `state`, though tools may pair this with a separate move to `validation`. |

### Accept review / request changes

| Aspect | Semantics |
| --- | --- |
| Required inputs | Task ID and review decision: accept, request changes, or reject. Optional reviewer and notes. |
| Files read | `.tandem/tandem.md`, target task document in `.tandem/board/`, and document index for core reference validation. |
| Files written | Target task document; append current actor event log under `.tandem/events/<actor_id>.jsonl`. |
| Validation/errors/warnings | Error if the ID is missing, resolves to a log document or non-task document, requested review status is not canonical, review object would be malformed, or existing core references are unresolved. Warn for unresolved related `references` or completion-policy issues when the review remains unaccepted. |
| Event | `review.accepted`, `review.changes_requested`, or `review.rejected`. |
| Resulting state | Task stays in `.tandem/board/`; `review.status` becomes `accepted`, `changes-requested`, or `rejected`; `decidedAt` is set; optional reviewer/notes are recorded. |

Requesting changes does not automatically set accord status to `rework` or move task state to `in-progress`; tools may offer that as a paired mutation, but protocol semantics keep review, accord, and state separate.

### Complete/archive

| Aspect | Semantics |
| --- | --- |
| Required inputs | Task ID and `completion.summary`. Optional files changed, validation result summary, reviewer/completer, and completion notes. |
| Files read | `.tandem/tandem.md`, target task document in `.tandem/board/`, `.tandem/logs/` destination index, and document index for validation. |
| Files written | Completed task document in `.tandem/logs/`; remove/move the active `.tandem/board/` task document; append current actor event log under `.tandem/events/<actor_id>.jsonl`. |
| Validation/errors/warnings | Error if the ID is missing, resolves to a non-task document, already lives only in logs, required structure is invalid, destination would duplicate an existing log path/ID, or `parentId`/`blockers` are unresolved. Warn, but allow completion, when `review.status` is not `accepted`, an existing accord is not `accepted`, related `references` or rule sources are unresolved, or optional completion metadata is malformed but recoverable. |
| Event | `task.completed`. |
| Resulting state | Task is archived as a Markdown log document in `.tandem/logs/` with `completedAt` and `completion.summary`; active board document is gone; log document is the completed-work source of truth. |

Completion is not a persistent board state. The completed log may omit active-only `state`, but must retain enough identity, completion, body, review, accord, relationship, reference, and retained legacy inline-checklist information to stand alone.

### Post-v0 restore/reopen boundaries

Restore/reopen is not part of the v0 command surface. The protocol reserves event names so future tooling can distinguish two concepts:

- `task.restored` — move a completed log document back to `.tandem/board/` while preserving completion history.
- `task.reopened` — mark previously completed work as needing new active work, likely with a new active `state` and follow-up context.

V0 tools should be able to read unknown future restore/reopen events as ordinary audit records, but do not need to implement restore/reopen mutations.

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

Using `tandem` as the working CLI binary name:

```text
tandem init
tandem list
tandem show <id>
tandem add --title ... --state todo --kind epic
tandem move <id> --state validation
tandem update <id> --kind epic --priority high --tag cli
tandem complete <id> --summary ...
tandem log list|show|search
tandem search <query>
tandem accord ready|claim|deliver|accept|rework|block|fail
tandem rules list|add|edit|delete
tandem decision list|show|add
tandem tui
```

This is protocol-facing command shape only. Detailed CLI/TUI behavior belongs in `../tandem/`. `tandem update` is the v0 metadata-edit path for active board tasks; it does not update completed logs, workflow `state`, or `parentId`.

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
Epic/theme/milestone         → Tandem task with `kind: epic` and child tasks linked by `parentId`
```

## Open protocol questions

All previously listed v0 protocol decision questions are now resolved. Remaining protocol work is specification detail, tracked in `todo.md`.
