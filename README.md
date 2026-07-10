# Tandem

[placeholder for tandem image]

[Website](https://trytandem.dev/) · [Quickstart](https://trytandem.dev/quick-start/) · [CLI guide](https://trytandem.dev/cli/) · [TUI guide](https://trytandem.dev/tui/) · [Protocol](https://trytandem.dev/protocol/)

Tandem is a protocol for planning, coordination, and delegation of tasks/work between humans and agents:
- It lives in git alongside your project.
- Files are markdown with some basic frontmatter, and json when more convenient/compact for the agent.
- It contains a CLI and TUI written in Rust.

This is a monorepo that houses the spec, the cli and tui, and the documentation and deployed site.

## Repository layout

```text
protocol/      Protocol source of truth and detailed specification
tandem/        Rust CLI and Ratatui TUI application
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
  --tag tandem-v0.4.2 --path tandem --locked
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

Initialization creates a `.tandem/` workspace containing active board documents, completed logs, lifecycle events, project rules, and configuration. The Markdown files remain the source of truth; use the CLI or TUI for structured updates.

[placeholder for .tandem directory structure]

See the [agent-first quickstart](https://trytandem.dev/quick-start/) to take a small task through the complete workflow.

## Everyday workflow

Human describes task to agent -> agent creates task and accord (contract with deliverables) -> human or agent orchestrator delegates the task to begin being worked on -> agent returns results when done to orchestrator -> work is either auto approved by meeting the requirements or, optionally a human gets final sign off -> task is completed, all work committed and cleaned up.

## TUI

[placeholder tui image/gif]

## Documentation

[docs on trytandem site]

## Extensions / Skills

[placeholder for pi extension]

If using claude code, codex, etc

[general guidance for agents]

## License

Tandem is available under the [MIT License](LICENSE).
