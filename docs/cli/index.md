---
title: CLI
description: Using the tandem command-line interface.
---

# CLI

The `tandem` binary is the canonical command-line interface for Tandem workspaces.

## Common commands

```sh
tandem init --title "My Project"
tandem add --title "Implement next slice"
tandem list
tandem show task-1
tandem move task-1 --state in-progress
tandem accord claim task-1 --assignee pi
tandem accord deliver task-1 --summary "Implemented and tested"
tandem complete task-1 --summary "Accepted and archived"
```

## Read commands

Read commands provide human-readable output by default and support `--json` envelopes for adapters:

```sh
tandem list --json
tandem show task-1 --json
tandem log list --json
tandem decision list --json
```

## Command families

V0 command families are `init`, `list`, `show`, `add`, `move`, `complete`, `log`, `search`, `accord`, `rules`, `decision`, and `tui`.

The CLI uses canonical command names and long flags only in v0.
