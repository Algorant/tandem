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
```

The default active states are `todo`, `in-progress`, and `validation`. Completion archives a task into logs instead of moving it to a permanent `done` state.

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
