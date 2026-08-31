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
  --tag tandem-v0.10.1 --path tandem --locked
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

## Migrating a workspace to protocol 0.3.0

Tested procedure (2026-08-31) for moving an existing pre-0.3.0 coordination
workspace to the protocol 0.3.0 binary. There is deliberately no
`upgrade`/`migrate` command, converter, or compatibility reader — the new
binary only opens 0.3.0 workspaces, so migration is owner-side and
semi-manual. This procedure is a working draft and may be removed once the
migration surface stabilizes.

1. **Install and verify the new binary.** `curl -fsSL https://trytandem.dev/install.sh | sh` then confirm `tandem --version` reports the 0.12.x release.
2. **Archive the old workspace in git.** Move the whole `.tandem` directory aside (for example `.tandem mv .tandem .tandem_old`), add the ignored actor identity (`printf '.tandem_old/actor-id\n' >> .gitignore`), and commit. The archived state stays recoverable and the board/log/papercut history is not destroyed.
3. **Initialize the new workspace.** `tandem init --title "..."` creates a minimal protocol 0.3.0 config (states, empty rule categories).
4. **Transpose durable records by recreating them.** The fresh workspace uses new IDs and mandatory Accord, so recreate rather than copy:
   - Active state: `tandem add task "..." --acceptance "criterion" [...]` for each retained Task (mandatory Accord acceptance). Add Papercut-tagged low-priority Tasks for friction notes. Old references and provenance can be dropped; a one-line "continued from task-N, archived under .tandem_old" note preserves context without dangling references.
   - Rules: `tandem rules add <category> "<text>"` per rule. Omit `--source` when it pointed at archived records.
   - Key ADR decisions: `tandem add decision "..." --body ...` then set `status: accepted` and `decidedAt` (at present the CLI cannot update decision documents — see below).
5. **Verify.** `tandem list`, `show`, `search`, `rules list`, `accord claim/deliver`, and the TUI should behave normally against the fresh workspace.

Defects surfaced by the first run of this procedure (tracked as papercut-tagged
Tasks and a fix Task in the workspace): `tandem update` rejects decision
documents; `add decision` wrote files to `.tandem/tasks` instead of
`.tandem/decisions` (place them there when this is seen); and rules are still
config-backed with flat ids instead of per-file composite-id records.
