---
title: CLI
description: Using the tandem command-line interface.
---
The `tandem` binary is the command-line interface for Tandem workspaces. Use it to initialize a workspace, manage Board tasks, record accord delivery evidence, validate work, complete logs, and provide JSON read output for adapters.

For a full end-to-end workflow, see the [Quickstart](/quick-start/).

## Install and initialize

Current source install with Cargo:

```sh
cargo install --git https://github.com/Algorant/tandem.git --tag tandem-v0.4.0 --path tandem --locked
tandem --version
```

From a local checkout:

```sh
cargo install --path tandem --locked
```

Initialize a repository once:

```sh
tandem init --title "My Project"
```

## Board tasks

```sh
tandem add --title "Write project brief" --description "Draft and validate the first docs slice."
tandem list
tandem show task-1
tandem move task-1 --state in-progress
tandem update task-1 --priority medium --tag docs --related-file docs/index.md
tandem update task-1 --body "$revised_body"
```

`tandem update --body` replaces an active Task's complete Markdown body exactly; pass an explicit empty string to clear it. Unchanged body or metadata values are no-ops and do not write a timestamp or event.

The default active states are `todo`, `in-progress`, and `validation`. Completion archives a task into logs instead of moving it to a permanent `done` state.

### Create and inspect the hierarchy

Create each document with the normal `add` command and use the ID returned by Tandem:

```sh
tandem add --title "Coordinate the release" --kind epic
# Tandem returns a global Epic ID, for example task-103.

tandem add --title "Write release notes" --parent task-103
# A direct Epic child is a global Task, for example task-104.

tandem add --title "Check upgrade notes" --parent task-104
# A direct Task child is a parent-derived Subtask, for example task-104-1.
```

The CLI—not an integration adapter—resolves each parent, derives the role, and allocates IDs across the Board and Logs. Epics and Tasks use global `task-N`; only a Subtask directly beneath a Task uses `<Task ID>-M`. Subtasks are leaves, so a child beneath `task-104-1` is rejected.

```sh
tandem show task-103       # includes its global Tasks in `tasks`
tandem show task-104       # includes active/logged work in `subtasks`
tandem list --parent task-104
tandem search "release" --parent task-104
```

A standalone Task can own Subtasks in the same way. A Task attached to a decision or custom document keeps a global ID and generic `parent` relationship, and it may also own Subtasks.

IDs are immutable. `tandem update <id> --parent <id>` is accepted only when the prospective relationship preserves the document's canonical role and ID. Reparenting a global Task beneath a Task would turn it into a Subtask, so Tandem rejects that mutation rather than renaming the ID or rewriting references. Parented Epics, nested Subtasks, hierarchical IDs directly beneath Epics, and global-ID Subtasks are structural errors with no legacy compatibility path.

The older `--subtask` inline-checklist option is deprecated; create a first-class Subtask document with `--parent` for new lifecycle-bearing checklist work.

## Accords and validation

```sh
tandem accord claim task-1 --assignee alice

tandem accord deliver task-1 \
  --summary "Drafted the brief and checked the rendered docs" \
  --deliverable "Updated docs/index.md" \
  --validation "Ran cd site && bun run check:docs" \
  --file-changed docs/index.md

tandem accord accept task-1 --reviewer bob --summary "Looks good"
# or: tandem accord rework task-1 --note "Please add the install path."
```

Claiming from `todo` moves a task to `in-progress`. Delivering moves it to `validation`. Workers should record evidence; reviewers or orchestrators decide acceptance, rework, blocking, or failure.

## Complete and read logs

```sh
tandem complete task-1 \
  --summary "Published the project brief" \
  --validation "Reviewed by Bob" \
  --file-changed docs/index.md

tandem log list
tandem log show task-1
tandem search "project brief"
```

The v0 CLI may warn when separate `review.status` metadata is missing. Treat that warning as a policy reminder and complete only when the responsible reviewer or orchestrator has intentionally accepted the work.

## Decisions and rules

Use decisions for durable choices and rules for workspace coordination expectations:

```sh
tandem decision list
tandem decision show decision-1
tandem rules list
```

Decision records are ADR-compatible Markdown documents. Rules are grouped as `always`, `never`, `prefer`, and `context`.

## JSON reads

Read commands provide human-readable output by default and support `--json` for adapters:

```sh
tandem list --json
tandem show task-1 --json
tandem search "validation" --json
tandem log list --json
tandem decision list --json
tandem rules list --json
```

## Open the TUI

```sh
tandem tui
```

The TUI is the daily Board view for tasks, validation, logs, rules, and decisions.

## Command families

V0 command families are `init`, `list`, `show`, `add`, `move`, `update`, `complete`, `search`, `log`, `accord`, `rules`, `decision`, `tui`, and `version` / `--version`.

The CLI uses canonical command names and long flags only in v0.
