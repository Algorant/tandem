# Tandem

[placeholder for tandem image]

[Website](https://trytandem.dev/) · [Quickstart](https://trytandem.dev/quick-start/) · [CLI guide](https://trytandem.dev/cli/) · [TUI guide](https://trytandem.dev/tui/) · [Web guide](https://trytandem.dev/web/) · [Workspace](https://trytandem.dev/workspace/)

Tandem is a protocol for planning, coordination, and delegation of tasks/work between humans and agents:
- It lives in git alongside your project.
- Files are markdown with some basic frontmatter, and json when more convenient/compact for the agent.
- It contains a CLI and TUI written in Rust.

This is a monorepo that houses the spec, the cli and tui, and the documentation and deployed site.

## Repository layout

```text
protocol/      Normative protocol source of truth and detailed specification
tandem/        Executable Rust protocol, project/app layers, CLI, and Ratatui TUI
docs/          Public documentation source
site/          Astro Starlight documentation site
extensions/    Agent and editor adapters, including pi-tandem
```
## Install

Install the latest released binary with the user-local, no-sudo installer:

```sh
curl -fsSL https://trytandem.dev/install.sh | sh
tandem --version
```

Release binaries are currently published for Linux and mac. Windows binaries are not published yet.

With Rust and Cargo, you can instead install the current tagged source:

```sh
cargo install --git https://github.com/Algorant/tandem.git \
  --tag tandem-v0.9.0 --path tandem --locked
```
### AUR
Released binary through `tandem-bin` in the AUR.

## Get started

Initialize Tandem once at the root of a project:

```sh
cd /path/to/your/project
tandem init
or optionally:
tandem init --title "My Project"
```

## [placeholder for workflow diagram]

Initialization creates a `.tandem/` workspace containing active Board documents, completed or canceled Logs, lifecycle events, project rules, and configuration. The first `tandem papercut add` lazily creates an optional searchable inbox for small, non-blocking friction. The Markdown files remain the source of truth; use the CLI or TUI for structured updates.

[placeholder for .tandem directory structure]

See the [agent-first quickstart](https://trytandem.dev/quick-start/) to take a small task through the complete workflow.

## Implementation architecture

The Markdown under [`protocol/`](protocol/) is normative. Its executable Rust
implementation lives in [`tandem/src/protocol/`](tandem/src/protocol/) and is
consumed by the concrete [`project::TandemProject`](tandem/src/project/mod.rs)
filesystem boundary. Shared [`app`](tandem/src/app/) operations coordinate
protocol validation and project I/O. [`cli`](tandem/src/cli/) and
[`tui`](tandem/src/tui/) are peer interfaces over those operations;
[`main.rs`](tandem/src/main.rs) only composes process startup and exit handling,
and `tui/mod.rs` wires the terminal application and cohesive TUI modules.

New projects use protocol `0.2.0`. A discovered `0.1.0` project requires an
explicit `tandem upgrade` before ordinary project operations; upgrades are not
implicit.

## Everyday workflow

Human describes task to agent -> agent creates task and accord (contract with deliverables) -> human or agent orchestrator delegates the task to begin being worked on -> agent returns results when done to orchestrator -> work is either auto approved by meeting the requirements or, optionally a human gets final sign off -> task is completed, all work committed and cleaned up.

Large outcomes use a strict Epic → Task → Subtask hierarchy:

```text
task-10       Epic (root `kind: epic`, global ID)
└── task-11   Task (direct Epic child, global ID, `epic-task`)
    └── task-11-1   Subtask (direct Task child, parent-derived ID, `subtask`)
```

Only Tasks are delegated initially. One Task worker owns its leaf Subtasks as a bounded execution checklist and returns one Task-level handoff; Epics and Subtasks are not independently delegated. Tandem rejects nested Epics, children beneath Subtasks, role/ID mismatches, and role-changing or ID-invalidating reparenting. See [Epics, Tasks, Subtasks, and related work](https://trytandem.dev/concepts/#epics-tasks-subtasks-and-related-work).

## TUI

[placeholder tui image/gif]

## Local read-only web view

Browse the nearest workspace with the bundled browser interface:

```sh
tandem web
```

It opens the default browser on an available `127.0.0.1` port. Use
`--port <port>` for a stable port or `--no-open` to print the URL only. The MVP
serves one workspace and has no mutations or remote access. See the
[web guide](https://trytandem.dev/web/) for views, security boundaries, and
deferred capabilities.

## Documentation

[docs on trytandem site]

## Extensions / Skills

[placeholder for pi extension]

If using claude code, codex, etc

[general guidance for agents]

## License

Tandem is available under the [MIT License](LICENSE).
