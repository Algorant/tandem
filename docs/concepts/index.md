---
title: Concepts
description: Core Tandem vocabulary and workflow model.
---
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

## Epics, subtasks, and related work

A first-class subtask is a full task document whose `parentId` points to another task. It has its own state, owner, accord, validation, blockers, and completion history. The parent can be an epic or an ordinary task. If `parentId` instead resolves to a decision or custom document, it is a valid generic parent relationship—not a subtask.

New children normally receive a parent-derived ID. For example, a child of `task-103` becomes `task-103-1`, and its child becomes `task-103-1-1`. Tandem allocates the next suffix across both the active Board and completed Logs, so completing a child does not make its ID available again.

`parentId` defines the hierarchy; the ID shape does not. Older children with flat IDs such as `task-137` remain valid. IDs are immutable, so reparenting changes `parentId` without renaming the task or rewriting links to it.

Choose each relationship for its meaning:

| Use | When |
| --- | --- |
| Epic (`kind: epic`) | A broad outcome groups several independently managed tasks. An epic is still a normal task. |
| Ordinary parent task | One task naturally breaks into tracked child work but does not need to be called an epic. |
| First-class child task | Work needs its own owner, state, accord, validation, blockers, or completion record. |
| `blockers` | Another document must be resolved before this task can proceed. |
| `references` | A decision, sibling task, log, or other document is useful context but not a dependency or parent. |
| Inline `subtasks` checklist | Preserve an older checklist already in a task. Inline items are legacy data and should not be created for new tracked work. |

Completed children move to Logs like any other task. They still count toward their parent's history and ID allocation, while active children remain on the Board.

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
