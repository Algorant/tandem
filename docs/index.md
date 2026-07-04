---
title: Welcome to Tandem
description: Local-first coordination for humans and agents.
---

# Tandem documentation

Tandem is a local-first coordination system for people and agents working in the same repository. It keeps tasks, work agreements, decisions, validation notes, and completed-work history in plain Markdown under `.tandem/`, then layers a CLI, TUI, and lightweight integrations on top.

The aim is durable shared context: project state that can be read in any editor, reviewed in Git, searched later, and trusted by both humans and automation.

## Why local-first Markdown?

Human/agent projects often lose important context in chat threads, private dashboards, or one-off tool state. Tandem keeps coordination data next to the code so it can move through normal repository workflows.

- **Readable by default** — every active task and decision is a Markdown file with YAML frontmatter.
- **Reviewable in Git** — state changes, decisions, and completed logs can be diffed like source code.
- **Friendly to many actors** — humans, coding agents, review agents, and editor tools can all operate on the same local files.
- **No hidden done pile** — completed work is archived into searchable logs with validation notes and changed files.

## The mental model

Tandem separates three ideas that are often collapsed in simpler task boards:

1. **Board state** — where the task sits in the workflow: `todo`, `in-progress`, or `validation`.
2. **Accord status** — the work agreement: ready, claimed, delivered, accepted, rework, blocked, or failed.
3. **History** — completed logs and decisions that explain what happened and why.

A typical loop is small and explicit:

```text
plan task → start/claim → deliver evidence → validate → complete into logs
```

That separation is especially important for agent work. A worker can deliver evidence without self-approving, while a human or orchestrator keeps responsibility for acceptance and completion.

## Get started

Run the full workflow in the [Quickstart](/quick-start/). It covers install options, `tandem init`, adding and claiming a task, delivering through an accord, validating and accepting, completing into logs, and opening `tandem tui`.

If you already have Tandem installed, the shape is:

```sh
tandem init --title "My Project"
tandem add --title "Write project brief" --description "Draft and validate the first docs slice."
tandem move task-1 --state in-progress
tandem accord claim task-1 --assignee alice
tandem accord deliver task-1 --summary "Drafted and checked" --validation "Ran docs check"
tandem accord accept task-1 --reviewer bob --summary "Looks good"
tandem complete task-1 --summary "Published the brief" --validation "Reviewed by Bob"
```

## Explore the docs

- [Quickstart](/quick-start/) — run one task through the full lifecycle.
- [Concepts](/concepts/) — understand board states, accords, epics, rules, decisions, and logs.
- [CLI](/cli/) — learn the core command families.
- [TUI](/tui/) — use the terminal interface for board work.
- [Protocol](/protocol/) — inspect the `.tandem/` file model.
- [Extensions](/extensions/) — connect optional agent/editor adapters.
- [Guides](/guides/) — follow practical maintenance workflows.

## Current status

Tandem is in early v0 implementation. The core CLI surface is implemented, the Ratatui TUI is growing around Board/Validation/Logs/Rules/Decisions workflows, and the docs site keeps canonical content in `docs/` so the public site can evolve with the protocol.
