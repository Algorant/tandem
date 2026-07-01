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

## Decision commands

Use `tandem decision` for durable decisions and ADR-compatible records:

```sh
body=$(cat <<'MD'
## Status

Accepted.

## Context

Why this choice is needed.

## Decision

What has been decided.

## Consequences

What changes because of it.

## Supersession

- Supersedes: none
- Superseded by: none
MD
)

tandem decision add --title "Use Tandem decisions for ADRs" --body "$body" --reference task-87 --tag adr
tandem decision list
tandem decision show decision-1 --json
```

`decision` documents do not use task workflow `state`. Put ADR status and supersession in body sections, add optional frontmatter metadata only when editing the Markdown record directly, and use `references` for tool-visible links.

## Command families

V0 command families are `init`, `list`, `show`, `add`, `move`, `complete`, `log`, `search`, `accord`, `rules`, `decision`, and `tui`.

The CLI uses canonical command names and long flags only in v0.
