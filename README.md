---
description: Local-first CLI, TUI and protocol for planning and delegating work between humans and agents, stored as Markdown in the repository.
---

# Tandem

Tandem helps people and agents plan, claim, deliver, review, and archive project work in a shared local board.

[Website](https://trytandem.dev/) · [Quickstart](https://trytandem.dev/quick-start/) · [Concepts](https://trytandem.dev/concepts/) · [CLI](https://trytandem.dev/cli/) · [TUI and web](https://trytandem.dev/tui/)

## Install

On Linux or macOS, install the latest release and verify the command:

```sh
curl -fsSL https://trytandem.dev/install.sh | sh
tandem --version
```

The installer is user-local and requires no `sudo`. With Rust, install from this repository:

```sh
cargo install --git https://github.com/Algorant/tandem.git --locked tandem
```

## How it works

Run `tandem init` in your project. Tandem creates an ignored `.tandem/` board of Markdown Tasks, Decisions, Rules, and completed Logs. Use the CLI or TUI to claim and deliver work; completed Tasks are archived in Logs. The [Quickstart](https://trytandem.dev/quick-start/) walks through one Task.

In a Git repository, the board belongs to the main worktree and syncs independently through the `tandem` branch. Source Git operations do not commit board changes. See [Concepts and sync](https://trytandem.dev/concepts/#board-storage-and-sync).

## Repository layout

```text
protocol/      Normative protocol source of truth
tandem/        Rust CLI, TUI, project, and app implementation
docs/           Public Markdown documentation source
site/           Astro Starlight site built from docs/
extensions/     Home for future agent and editor integrations
```

The public site is generated from Markdown under `docs/`; `site/src/content/docs/` is generated and must not be edited directly. The repository protocol in `protocol/` is normative; its executable Rust implementation is under `tandem/src/protocol/`.

## Development

- `just dev` builds the current checkout in release mode and opens a disposable Git-backed TUI sandbox.
- `just dev-project` explicitly opens the real project workspace.
- `just dev-check` runs native tests and a Git-backed workflow smoke test.
- `just site-build` builds the documentation site.

See [`tandem/README.md`](tandem/README.md) for CLI/TUI development and [`tandem/RELEASE.md`](tandem/RELEASE.md) for maintained release procedures.

Tandem is available under the [MIT License](LICENSE).
