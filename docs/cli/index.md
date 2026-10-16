---
title: CLI
description: Compact reference for Tandem commands and global options.
---

The `tandem` command finds the nearest workspace from the current directory. Run `tandem --help` or `tandem <command> --help` for the authoritative syntax.

**Upgrade together:** Move every machine and agent sharing a board to 0.17.1 together. Version 0.17.0 skips the new data-only audit events, so its timelines and attempt counts can be incomplete. Verify the binary each agent uses with `tandem --version`, and restart any still-running old clients. No board migration or event-file edits are needed.

| Command | Use |
| --- | --- |
| `init [--title TITLE]` | Create a board in the current project. |
| `add task TITLE --acceptance TEXT` | Create a Task; repeat `--acceptance` for more criteria. |
| `add decision TITLE` | Record a durable decision. |
| `show ID` · `assignment ID` | Read a record or a Task's delegation details. |
| `list` · `search QUERY` | Find Board items; use `--scope archived` for Logs. |
| `update ID` | Change supported metadata or a record body. Does not change Task state or Accord status. |
| `accord claim ID --assignee NAME` | Claim work. |
| `accord deliver ID --summary TEXT --evidence TEXT` | Record the result and evidence. Repeat `--evidence` as needed. |
| `accord rework ID --note TEXT` · `accord block ID --note TEXT` · `accord resume ID` · `accord release ID --note TEXT` · `accord fail ID --note TEXT` | Manage an Accord. |
| `review ID --criterion TEXT --note TEXT` | Send an acceptance criterion for human validation. |
| `complete ID` · `cancel ID --note TEXT` | Accept delivered work into Logs or cancel a Task. |
| `link add\|remove ID TYPE TARGET` | Manage typed Task links. |
| `sync` · `sync status` · `sync resolve ID --keep local\|remote\|edited` | Synchronize and inspect or resolve board changes. |
| `migrate [--dry-run]` | Convert a supported older board to the current protocol. |
| `rules list\|add\|edit\|delete` | Read or manage project rules. |
| `tui [--add-workspace PATH ...]` | Open the TUI; repeat `--add-workspace` to seed more local boards for this invocation. |
| `web [--port PORT] [--no-open]` | Start the local, read-only web view. |

Global flags: `-j, --json` for JSON, `-h, --help` for help, and `-V, --version` for the installed binary version. There are no other short flags.

A Task validation beginning with `$ ` is a runnable command; unprefixed text is a manual check. For example, `--validation '$ just dev-check'` records a command to run, while `--validation 'Review the rendered page'` records a manual check.

With `--json`, commands return an envelope such as:

```json
{"ok":true,"data":{},"warnings":[]}
```

JSON errors instead return `ok: false` and an `error` object (no `data` or `warnings`): `{"ok":false,"error":{"code":"...","message":"...","details":{}}}`. Human-readable output is the default. For a Task workflow, see the [Quickstart](/quick-start/).
