---
title: Concepts
description: Core Tandem vocabulary and workflow model.
---

# Concepts

Tandem is easiest to understand from the Board. A task starts in `todo`, moves to `in-progress` when someone starts or claims it, moves to `validation` when work is delivered, and is archived into logs when accepted work is completed.

The CLI, TUI, and integrations all operate on the same local Markdown files under `.tandem/`.

## Board task states

Task workflow state answers: **where is this task right now?**

The default active states are:

- `todo` — planned work that is not actively owned.
- `in-progress` — work that has been started or claimed.
- `validation` — delivered work waiting for acceptance, rework, blocking, failure, or completion.

Completion is not a permanent `done` column. `tandem complete` archives accepted work into `.tandem/logs/` so the active board stays focused on work that can still change.

Older workspaces may contain `review`; v0 readers treat it as a legacy alias for the preferred `validation` state where appropriate.

## Accords

An accord is the explicit work agreement for a task. It answers: **who took responsibility, what did they deliver, and what happened to that delivery?**

Common accord statuses are:

- `ready` — the task is ready to be claimed.
- `claimed` — an actor has taken responsibility.
- `delivered` — work was submitted with summary, deliverables, validation evidence, and changed files.
- `accepted` — a reviewer or orchestrator accepted the delivery.
- `rework` — changes were requested.
- `blocked` — progress is blocked by an external condition.
- `failed` — the scoped work cannot be completed as agreed.

Accord actions can synchronize Board state. Claiming a `todo` task moves it to `in-progress`; delivering moves it to `validation`; requesting rework from validation moves it back to `in-progress`.

This keeps agent workflows honest: a child worker can deliver evidence, while a human or parent orchestrator decides whether to accept, request rework, block, fail, or complete the task.

## Epics and task relationships

An epic is a normal task with `kind: epic`, not a separate document type. Use epics to group related work while keeping child tasks independently claimable and reviewable.

Relationship fields have different meanings:

- `parentId` — strict hierarchy, usually epic to child task.
- `blockers` — hard dependencies that must resolve before work can proceed.
- `references` — loose related context such as decisions, sibling tasks, or completed logs.
- `subtasks` — lightweight checklist items inside one task.

Create a child task when work needs its own owner, accord, validation, or blockers. Use subtasks when a checklist inside one task is enough.

## Rules

Rules are workspace coordination expectations stored in `.tandem/tandem.md`. They are grouped as `always`, `never`, `prefer`, and `context`.

Rules help humans and agents align before work starts. For example, a project can record validation expectations, tag conventions, or delegation policies. Use `tandem rules list` to inspect them.

## Decisions

Decision documents record durable product, architecture, or project choices. They are ADR-compatible by convention: use `type: decision`, keep a clear title, and structure the body with sections such as Status, Context, Decision, Consequences, and Supersession when useful.

Decisions do not use task workflow state. They remain active records that tasks can reference.

## Logs

Logs are completed task documents stored in `.tandem/logs/`. They preserve the task body, completion summary, validation notes, changed files, relevant accord metadata, and event context.

Use logs when you need to answer “what changed?”, “why was it accepted?”, or “what evidence did we have?” later:

```sh
tandem log list
tandem log show task-1
tandem search "validation"
```

## Workspace files

A Tandem workspace is a repository with a `.tandem/` directory:

```text
.tandem/
├── tandem.md        # workspace config, workflow states, and rules
├── board/           # active task and decision documents
├── logs/            # completed task history
├── events/          # per-actor lifecycle event logs
└── events.jsonl     # legacy global event log, still readable during transition
```

Active tasks and decisions are Markdown files with YAML frontmatter. The files are the source of truth; the CLI and TUI provide safe, structured operations over them.

## The daily loop

1. Read the Board in `tandem tui` or with `tandem list`.
2. Add or inspect a task.
3. Start and claim it.
4. Deliver summary, evidence, validation, and changed files through the accord.
5. Validate the result.
6. Accept and complete it into logs, or request rework and continue.

That loop is small enough for a human, a terminal workflow, or an agent orchestrator to follow without hiding the project record outside the repository.
