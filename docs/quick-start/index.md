---
title: Quickstart
description: Install Tandem and run one task from idea to completed log.
---
This guide takes you through the Tandem workflow in six steps. You can work with Tandem directly in the terminal or ask your agent to carry out the work.

## 1. Install Tandem

Choose one installation method.

### Installer

```sh
curl -fsSL https://trytandem.dev/install.sh | sh
```

### Rust

```sh
cargo install --git https://github.com/Algorant/tandem.git --tag tandem-v0.4.0 --path tandem --locked
```

### AUR

```sh
paru -S tandem-bin
```

## 2. Initialize a workspace

At the root of the project you want to coordinate, run:

```sh
tandem init
# Optional: --title "my tandem project"
```

Tandem creates a `.tandem/` workspace for your active tasks, completed work, and project coordination rules.

## 3. Create a task

For task creation, see the forthcoming [Guidance for agents](/guides/agents-and-adapters/) page. Direct your agent to create a task for you. The agent understands Tandem's fields and can fill them from a natural-language request, such as:

> help me research the best static site generators
>
> build me a simple terminal todo app

Run `tandem list` to view your tasks in the terminal, or ask your agent to show them.

## 4. Start the work

Invite your agent to begin with an actionable prompt:

> Please begin work on task-4

You can also ask your agent to delegate the task or ask what to do next. The agent can inspect the task, follow the workspace guidance, and report its progress.

## 5. Verify the result

When the work is ready, your agent may respond with a completion message such as:

> Task done! Please review the result: http://localhost:4321

Open the URL, inspect the changes, or review the deliverables that the agent reports. Confirm that the result meets your needs before you give feedback.

## 6. Give feedback

Tell your agent what needs to change. For example:

> The UX needs some work.

> The “Create task” button is not functioning.

The agent can make fixes and ask you to review the result again. When the work is validated, tell the agent that it is validated and move to the next task.

For more information about completed work, open the [Logs view in `tandem tui`](/tui/#views). For the wider workflow, see [Concepts](/concepts/), [CLI](/cli/), [TUI](/tui/), and [Extensions](/extensions/).
