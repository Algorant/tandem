---
title: Workspace
description: The local .tandem/ board, its files, and how it syncs between machines.
---

A Tandem workspace is the `.tandem/` directory in a repository. It stores active work, completed history, project rules, and the event record used by the CLI, TUI, and integrations. New to Tasks, Epics, Rules, or Decisions? Start with [Concepts](/concepts/).

## Layout

```text
.tandem/
├── tandem.md               # workspace settings
├── tasks/                  # active Tasks, Epics, and Subtasks
│   ├── task-196.md         # Epic
│   ├── task-197.md         # Task (parentId: task-196)
│   └── task-197-1.md       # Subtask (parentId: task-197)
├── decisions/              # durable Decisions
│   └── decision-3.md
├── rules/                  # one Rule per file
│   └── always-1.md
├── logs/                   # completed, canceled, and failed Tasks
└── events/                 # one audit ledger per checkout
    └── <actor-id>.jsonl
```

Files are plain Markdown with YAML frontmatter, stored flat within each directory; `parentId` connects Epics, Tasks, and Subtasks. Edit records with Tandem commands or in any editor.

## Records

Every record has an `id` such as `task-197` and a permanent `uid`:

```markdown
---
id: task-197
uid: 3f9a2c1d-8b6e-4c7a-9e2f-1d0b5a7c6e44
type: task
title: Rewrite Concepts page
state: in-progress
parentId: task-196
accord:
  status: "claimed"
  acceptance: ["The page explains Board states"]
---
```

The `uid` identifies the record on every machine and never changes. The `id` is the readable sequential number. A record created while it cannot sync yet gets a temporary ID such as `task-new-3f9a2c1d` and receives its number the first time it syncs; commands accept the temporary ID afterwards too. Do not change `id` or `uid` by hand.

## `tandem.md`

`tandem.md` holds the protocol version, the permanent `workspaceId`, the title, and the workflow states.

## `rules/`

Each Rule is one file named by its ID, such as `always-12.md`. Its category is `always`, `never`, `prefer`, or `context`, and the rule text is the Markdown body. Use `tandem rules list` to read them.

## `logs/`

`logs/` holds completed, canceled, and failed Tasks moved from `tasks/`. A Log keeps the full record plus `archivedAt` and a `resolution` with its outcome.

## `events/`

Each checkout appends an audit trail of its changes to its own ledger. Events are history, not the source of truth for records.

## Sync between machines

In a Git repository the board is not part of your source commits. `.tandem/` is ignored by the source branch and synchronized through the repository's `tandem` branch on the same remote:

- Tandem syncs after every change, refreshes before reads that are more than a minute old, and syncs in the background of the TUI and web view. `git pull` and `git push` of your code are unaffected.
- Changes from different machines combine record by record. Only a contradiction on the same record (for example two different titles, or completing a Task on one machine while editing it on another) waits for you: see `tandem sync status` and `tandem sync resolve`.
- Offline changes are saved and sync on the next use.
- A clone uses one board for all its worktrees. A fresh clone downloads the board on its first Tandem command.
- Tandem keeps a local safety copy of the board inside `.git`, so unsynced changes survive `git clean` or checking out an older commit.

Each checkout's identity lives in its Git directory (`.git/tandem-actor-id`), never in `.tandem/`. A board outside Git is a plain local folder that does not sync.

Moving an existing repository to this model: see [Upgrading to independent sync](/guides/upgrading-to-independent-sync/).
