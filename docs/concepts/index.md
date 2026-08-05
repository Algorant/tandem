---
title: Concepts
description: Core Tandem vocabulary and workflow model.
---
Tandem keeps work, agreements, expectations, and history visible in one local workspace. Start with the [Quickstart](/quick-start/) to create a workspace, then use the [Workspace](/workspace/) guide to learn how its pieces fit together.

## A few questions Tandem keeps visible

- **What needs to happen?** Tasks, Epics, and Subtasks describe and organize work.
- **Who agreed to do it?** Accords make ownership, delivery, validation, and acceptance explicit.
- **How should it be done?** Rules provide workspace coordination expectations.
- **What happened?** Decisions, Logs, and events preserve project history.

## The active task lifecycle

A task starts in `todo`, moves to `in-progress` when someone starts or claims it, and moves to `validation` when work is delivered. A reviewer or orchestrator can accept the delivery and complete the task, request rework, block it, or record a failure. Completion archives the task in Logs instead of leaving a permanent `done` state on the active Board.

This lifecycle keeps current work separate from completed history while making review explicit. The [CLI Reference](/cli/) documents each command, and the [TUI](/tui/) provides the same workflow through an interactive interface.

## How work is organized

Tandem uses a simple relationship between three kinds of work:

```text
task-103       Epic
└── task-104   Task
    └── task-104-1   Subtask
```

- An **Epic** is a broad outcome that groups related Tasks.
- A **Task** is independently delegated work. It can stand alone or belong to an Epic.
- A **Subtask** is a smaller, lifecycle-bearing item owned through its parent Task. It is not independently delegated.

Epics and Tasks use the global `task-N` namespace. A direct Epic child is therefore a Task such as `task-104`. Only a direct child of a Task is a Subtask, using the parent-derived `task-N-M` form. Subtasks are leaves and cannot have children.

## Accords

An accord is the explicit work agreement for a Task. It makes responsibility and review visible rather than implicit.

- **Purpose:** record who claimed the work, what they delivered, and the evidence for that delivery.
- **Use:** move an accord through `ready`, `claimed`, `delivered`, `accepted`, `rework`, `blocked`, or `failed` as the work changes.
- **Why it matters:** a worker can deliver evidence, while a human or orchestrator retains the decision to accept, request rework, or complete the Task.

Claiming a `todo` Task moves it to `in-progress`; delivering moves it to `validation`. See [Agents and adapters](/guides/agents-and-adapters/) for the framework-neutral accord contract.

## Rules

Rules are workspace coordination expectations stored in `.tandem/tandem.md`. They help people and agents apply the same repository policy.

- **Purpose:** define expectations such as validation, tagging, or delegation policy.
- **Use:** classify each rule as `always`, `never`, `prefer`, or `context`.
- **Why it matters:** applicability and strength stay explicit, so a narrow directive or prohibition is not mistaken for a general preference.

Use `tandem rules list` to inspect the active rules before work starts.

## Decisions

Decision documents preserve durable product, architecture, or project choices.

- **Purpose:** explain why the project chose a direction, not merely what a Task changed.
- **Use:** create a `type: decision` record with Status, Context, Decision, Consequences, and Supersession sections when useful.
- **Why it matters:** Tasks can reference a decision, and future work can understand the context without reconstructing it from conversation.

Decisions do not use task workflow state. They remain active records until superseded or deprecated.

## Logs and events

Logs are completed or canceled Task documents stored in `.tandem/logs/`. They preserve the Task body, summary, validation notes, changed files, accord metadata, and event context. Cancellation records a reason and remains auditable rather than deleting the file.

Events are append-only lifecycle records. Per-actor event files live in `.tandem/events/`; the legacy `.tandem/events.jsonl` file remains readable during transition. Together, Logs and events answer “what changed?”, “why was it accepted?”, and “what evidence did we have?”

```sh
tandem log list
tandem log show task-1
tandem search "validation"
```

## Workspace files

A Tandem workspace is a repository with a `.tandem/` directory:

```text
.tandem/
├── tandem.md        # workspace config and rules
├── board/           # active tasks and decisions
├── logs/            # completed task history
├── events/          # per-actor lifecycle event logs
└── events.jsonl     # legacy global event log
```

Active tasks and decisions are Markdown files with YAML frontmatter. The files are the source of truth; the CLI and TUI provide safe, structured operations over them.

## The daily loop

1. Read the Board with `tandem tui` or `tandem list`.
2. Add or inspect a Task.
3. Start and claim it.
4. Deliver a summary, evidence, validation, and changed files through the accord.
5. Validate the result.
6. Accept and complete it into Logs, or request rework and continue.

For integration-specific guidance, see the [Workflows](/guides/) guides.
