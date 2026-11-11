# tandem

The rust CLI and TUI for a tandem board, and the implementation behind the public documentation.

- `src/protocol/` — the executable form of [`protocol/README.md`](../protocol/README.md); normative meaning stays in the protocol
- `src/project/` — `.tandem/` discovery, files, Git sync and locking
- `src/app/` — shared operations over the project: Tasks, Accords, review, rules, queries
- `src/cli/`, `src/tui/`, `src/web.rs` — peer interfaces; none owns protocol rules
- `examples/themes/` — `default-dark` and `verdigris` theme files

The user-facing behavior lives in the [docs](../docs/): [CLI](../docs/cli/index.md), [TUI and web](../docs/tui/index.md), [configuration](../docs/reference/configuration.md), and [concepts](../docs/concepts/index.md). `tandem --help` is the authoritative command tree.

## Build and run

```text
cargo build --release
cargo run -- --version
cargo run -- init
cargo run -- list
```

From the repository root, `just dev` builds a release binary and opens a disposable Git-backed board; `just dev-check` runs the native checks and a workflow smoke test.

## Tests

The maintained tests are end-to-end: they drive the real binary against disposable Git repositories. Compiler, formatting and lint checks are separate.

```text
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

TUI changes are verified by running the real TUI in an hc terminal and reading the screen; see the repository [`AGENTS.md`](../AGENTS.md).

## Release

[`RELEASE.md`](RELEASE.md) holds the maintained release procedure, gates and installer targets. [`AUR.md`](AUR.md) covers the `tandem-bin` package.
