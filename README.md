![tandem](docs/banner.svg)

tandem is my local-first board for planning and delegating work between people and agents, stored as Markdown in the repository. It is a rust CLI and TUI, the protocol the board follows, and the docs that explain both.

## Install

```sh
curl -fsSL https://trytandem.dev/install.sh | sh
tandem --version
```

The installer is user-local and needs no `sudo`. With rust, install from this repository:

```sh
cargo install --git https://github.com/Algorant/tandem.git --locked tandem
```

## How to use this repo

Do not clone or build it to use tandem. Install the release above, then read the [quickstart](https://trytandem.dev/quick-start/) or [concepts](https://trytandem.dev/concepts/), and point your agent at them, or at one part of this repo.

## Core

- **[protocol](protocol/README.md)** — the normative board format: Tasks, Decisions, Rules, Logs, Accords, and the event ledgers behind them
- **[tandem](tandem/README.md)** — the rust CLI and TUI that read and write a `.tandem/` board, with the app and project layers underneath
- **[docs](docs/)** — the published source for [trytandem.dev](https://trytandem.dev/), built by [site](site/)

## Development

- `just dev` builds a release binary and opens a disposable Git-backed board
- `just dev-check` runs the native checks and a workflow smoke test
- `just site-build` builds the documentation site

## License

[MIT](LICENSE)
