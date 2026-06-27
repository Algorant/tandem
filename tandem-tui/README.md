# Tandem CLI/TUI

This directory contains planning and implementation work for the Tandem user-facing CLI and terminal UI.

Current phase: CLI v0 surface complete for the current known scope, with forward focus on the Rust/Ratatui TUI. `tdm tui` now launches a minimal Board-first shell on top of the same protocol concepts.

## Scope

The CLI/TUI area owns:

- `tdm` CLI command design and user experience
- CLI output/error conventions and command workflow design
- Ratatui app architecture
- board/review/logs/rules/decisions views
- responsive layouts
- theming
- mouse support and hit-map interaction model
- keyboard and command-palette UX
- review, accord, completion, and logs workflows as presented in the UI
- TUI tests and snapshots

The CLI/TUI area does **not** own the underlying protocol semantics. Protocol rules and data-model decisions belong in `../protocol/`, though the CLI and TUI must represent them faithfully.

## Current status

Planning/specification plus implementation mode. A Rust binary package now lives in this directory and builds a `tdm` binary with `init`, `list`, `show`, `add`, `move`, `complete`, `search`, read-only `log`, `accord ready|claim|deliver|accept|rework|block|fail`, `rules list|add|edit|delete`, and `decision list|show|add` coverage. The current known CLI surface is considered complete unless new feature requests or bugs appear. Frontmatter reads use the approved `yaml-rust2` dependency while command mutations use raw-source, minimal-diff patches. Completion writes nested `completion` metadata, accord actions write canonical validation/timestamp metadata, and read paths tolerate earlier flat completion fields. The initial `tdm tui` implementation uses Ratatui plus crossterm to render a read-only Board-first shell with state/item navigation, selected-item details, reload, keyboard quit, and basic mouse wheel/click handling. Board mutations and the Review/Logs/Rules/Decisions views are the next implementation focus.

## Build/run

From this directory:

```text
cargo run -- init --title "Demo"
cargo run -- add --title "Implement next CLI slice"
cargo run -- list
cargo run -- move task-1 --state in-progress
cargo run -- accord ready task-1 --assignee pi --validation "cargo test"
cargo run -- complete task-1 --summary "Implemented and tested"
cargo run -- log list
cargo run -- rules add --category always --rule "Run tests before completing tasks."
cargo run -- tui
```

Use `cargo run -- <command>` during early development. The package binary name is `tdm`.

## Documentation

- `plan/spec.md` — CLI/TUI draft
- `plan/todo.md` — CLI/TUI task tracker
- `../README.md` — parent project overview
- `../plan/spec.md` — parent project plan
- `../plan/todo.md` — parent project todo
- `../protocol/README.md` — protocol area overview
- `../protocol/plan/spec.md` — protocol draft the CLI/TUI must follow
- `../AGENTS.md` — agent guidance and documentation sync rules

## Sync requirements

This directory must stay aligned with the parent Tandem docs and the protocol docs.

When CLI/TUI architecture, invocation, layout, workflow, or status terminology changes, update all affected docs in the same change:

- `../README.md`
- `../plan/spec.md`
- `../plan/todo.md`
- `README.md`
- `plan/spec.md`
- `plan/todo.md`
- `../protocol/README.md` or `../protocol/plan/spec.md` if protocol-facing behavior changes
- `../AGENTS.md` if agent rules or workflows change

No drift is allowed. If this README contradicts parent or protocol docs, fix the contradiction immediately.

## Key current decisions

- Product/protocol name: **Tandem**
- CLI/TUI directory: `tandem-tui/`
- CLI binary: `tdm`
- CLI design and the current known CLI v0 implementation came before TUI implementation; future CLI work should be explicit new features or bug fixes.
- V0 TUI invocation: `tdm tui` only.
- TUI implementation target: Rust + Ratatui with crossterm terminal events/backend.
- `tdm tui` currently starts with a Board-first read-only shell; broader MVP views and mutations are the active focus.
- Basic feature parity with live Brainfile CLI/TUI is the baseline; improvements and omissions must be intentional.
- Do not assume a persistent `done` column.
- Make review, accord status, validation, and logs prominent.
- Theme support is required from the beginning.
- Mouse support should use a hit-map style model, be enabled by default, and exclude drag/drop in v0.


## Locked v0 CLI/TUI decisions

- v0 commands: `init`, `list`, `show`, `add`, `move`, `complete`, `log`, `search`, `accord`, `rules`, `decision`, `tui`.
- `tdm log`: `list`, `show`, `search`.
- `tdm rules`: `list`, `add`, `edit`, `delete`.
- `tdm accord`: `ready`, `claim`, `deliver`, `accept`, `rework`, `block`, `fail`.
- Human-readable output by default: compact tables for list/search and labeled detail blocks for show/log/decision.
- All read commands support `--json` using `{ "ok": true, "data": ..., "warnings": [] }` envelopes.
- V0 CLI uses canonical command names and long flags only; no short aliases.
- First implementation language: Rust inside `tandem-tui/`.
- `tdm decision`: `list`, `show`, `add`.
- First TUI MVP: board mutations immediately; Board, Review, Logs, Rules, Decisions views; theme and mouse support included.
- Review queue: simple filtered list in v0.
- Keymaps: fixed defaults in v0; custom keymap config later.
- Markdown rendering: styled basics in v0.
- Theme config loading order: built-in defaults, then user TOML themes in `~/.config/tandem/themes/*.toml`, then workspace TOML override at `.tandem/theme.toml`.
- Deferred from v0: non-core command families and integrations listed in `plan/spec.md`, plus schemas, fixtures, and root Rust workspace layout.
