---
title: Workspace
description: The local .tandem/ workspace and its files.
---

A Tandem workspace is the `.tandem/` directory in a repository. It stores active work, completed history, project rules, and the event record used by the CLI, TUI, and integrations. New to Tasks, Epics, Rules, or Decisions? Start with [Concepts](/concepts/).

## Layout

```text
.tandem/
├── tandem.md                         # workspace config and rules
├── board/                            # active Markdown documents
│   ├── task-196.md                   # Epic
│   ├── task-197.md                   # Task (parentId: task-196)
│   ├── task-197-1.md                 # Subtask (parentId: task-197)
│   └── decision-3.md                 # Decision
├── logs/                             # completed or canceled documents
├── actor-id                           # local checkout identity
├── events/                            # timestamped event logs
│   └── <actor_id>.jsonl
└── events.jsonl                       # legacy global event log
```

The files are stored flat within each directory. When tasks have a hierarchy, `parentId` connects the documents. The files use Markdown with YAML frontmatter and are the source of truth; `tandem` provides safe structured operations over them.

## `tandem.md`

`tandem.md` defines workspace settings and repository coordination policy. It can declare workflow states, defaults, and structured rules for humans and agents.

### Rules

Rules are grouped by category. An `always` rule requires an action or invariant whenever it applies. A `never` rule prohibits an action or outcome.

```yaml
rules:
  always:
    - id: 12
      rule: "Run the documentation checks before delivery."
      source: "task-199"
  never:
    - id: 13
      rule: "Do not commit generated site content."
```

The other categories are `prefer` for a default choice and `context` for information that does not command an action. Use `tandem rules list` to inspect the complete set.

## `board/`

`board/` contains active Task, Subtask, and Decision documents. For example, `task-197.md` can be a Task, `task-197-1.md` its Subtask, and `decision-3.md` a durable product or architecture choice. The filenames use canonical IDs; titles and hierarchy come from document frontmatter.

Each document combines structured YAML frontmatter with a human-readable Markdown body. Tasks use workflow state; Decisions are durable records and do not use task workflow state.

## `logs/`

`logs/` contains completed or canceled work archived from the Board. A log keeps the original task context together with its completion summary, validation evidence, and changed files.

A completed log has the same Markdown/YAML shape as an active document:

```markdown
---
id: task-104
type: task
title: Rewrite Concepts page
completedAt: 2026-02-20T14:30:00Z
completion:
  outcome: completed
  summary: Published the revised Concepts page.
  validation: bun run check:docs
filesChanged:
  - docs/concepts/index.md
---

## Outcome

The page now explains the Board workflow and workspace files.
```

Canceled work uses `completion.outcome: canceled` and keeps its reason for later audit.

## `actor-id`

`actor-id` identifies the current independent checkout or linked worktree. Tandem creates and reuses this local UUID automatically. It is ignored in Git projects and is not an integration setting.

## `events/`

A timestamped log of everything that happens within a Tandem project.
