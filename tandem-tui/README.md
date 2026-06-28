# Tandem CLI/TUI

This directory contains planning and implementation work for the Tandem user-facing CLI and terminal UI.

Current phase: CLI v0 surface complete for the current known scope, with forward focus on the Rust/Ratatui TUI. `tdm tui` now launches a Board-first shell with a real read-only Review queue on top of the same protocol concepts.

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

Planning/specification plus implementation mode. A Rust binary package now lives in this directory and builds a `tdm` binary with `init`, `list`, `show`, `add`, `move`, `complete`, `search`, read-only `log`, `accord ready|claim|deliver|accept|rework|block|fail`, `rules list|add|edit|delete`, and `decision list|show|add` coverage. The current known CLI surface is considered complete unless new feature requests or bugs appear. Frontmatter reads use the approved `yaml-rust2` dependency while command mutations use raw-source, minimal-diff patches. Completion writes nested `completion` metadata, accord actions write canonical validation/timestamp metadata, and read paths tolerate earlier flat completion fields. The current `tdm tui` implementation uses Ratatui plus crossterm to render top-level Board, Review, Logs, Rules, and Decisions tabs with built-in `default-dark` styling, selectable built-in `verdigris` styling, and workspace theme overrides from `.tandem/theme.toml`. `1`..`5` are the keyboard top-level view switchers, with mouse tab clicks still available as explicit pointer navigation. Board uses Brainfile-style state subviews rather than simultaneous narrow columns: configured states render as count tabs, the active state gets the full Board list area, rows show richer priority/title/accord/review/checklist/tag/assignee/update/path metadata, and selected-item details remain below. Board keeps local state/item navigation, reload, keyboard quit, basic mouse wheel/click handling, and first Board mutations: `a` quick-adds a task in the selected/default configured state, while `H`/`L` moves the selected task to the previous/next configured state and reloads the board. Review renders a real read-only filtered queue with local list/detail navigation, reason badges, accord/review/state/priority metadata, blockers, and action hints for CLI accord/complete flows. Logs has a real completed-work browser: recency-sorted list, selection/navigation, local list/detail focus, detail pane with completion metadata/body/path/event context, safe load warnings, and `/` search filtering across log IDs, titles, summaries, bodies, validation text, and files. Rules lists project rules grouped by `always`/`never`/`prefer`/`context` and supports local category navigation plus add/edit/delete prompts from `src/tui/rules.rs`. Decisions lists active decision documents, supports local list/body focus, shows selected decision metadata/body/path, and supports a basic title/body add prompt from `src/tui/decisions.rs`. `h/j/k/l` stay inside the active view, and Tab/BackTab cycle focus only where a view has meaningful focusable panes. Remaining gaps include user theme discovery from `~/.config/tandem/themes/*.toml`, additional Board mutations, Review action buttons/mutations, Decisions references/tags prompt parity, hot reload, and richer action buttons.

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

## Implemented TUI themes and keys

`tdm tui` starts from the built-in `default-dark` palette. If `.tandem/theme.toml` exists, it may select another built-in base and then override simple TOML-style string color values (`"#RRGGBB"`, `"#RGB"`, or supported terminal color names):

```toml
base = "verdigris"
name = "verdigris"
```

Supported built-in bases are `default-dark` and `verdigris`; `builtin` and `extends` are accepted aliases for `base`.

- root keys: `base`, `builtin`, `extends`, `name`
- `[colors]`: `background`, `panel`, `text`, `muted`, `accent`, `success`, `warning`, `error`, `border`, `selected_bg`, `selected_fg`
- `[priority]`: `critical`, `high`, `medium`, `low`, `none`
- `[badges.accord]`: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`, `unknown`
- `[badges.review]`: `not-ready`, `pending`, `accepted`, `changes-requested`, `rejected`, `failed`, `unknown`

To evaluate Verdigris visually from the repository root:

```text
cp tandem-tui/examples/themes/verdigris.toml .tandem/theme.toml
cd tandem-tui
cargo run -- tui
```

In the TUI, use `1`..`5` to switch Board/Review/Logs/Rules/Decisions, arrow keys or `j`/`k` to move, `/` in Logs to filter, `?` for help, and `q` to quit. `h/l` stays local: Board state subviews, Review/Logs/Decisions list-detail focus, and Rules categories. Tab/BackTab cycles focus only in views with focusable panes; in Rules it stays in view and shows a hint instead of switching top-level views. A manual PTY smoke should confirm the status line includes `theme built-in verdigris + .../.tandem/theme.toml`, the warm vellum text remains readable on dark panels, priority/accord/review badges use diff-red, ochre, aqua, moss, fern, and patina intentionally, and the keyboard focus semantics above hold across views 1..5. Delete `.tandem/theme.toml` to return to `default-dark`.

`NO_COLOR=1` or `TANDEM_NO_COLOR=1` uses the terminal/no-color fallback even when `.tandem/theme.toml` selects Verdigris. Loading user theme files from `~/.config/tandem/themes/*.toml` is still planned.

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
- `tdm tui` currently has top-level Board, Review, Logs, Rules, and Decisions tabs. `1`..`5` are the keyboard view switchers, mouse tab clicks switch views explicitly, and local navigation keys stay inside the active view. Board has state subview tabs with counts, a full-width selected-state list with richer rows, `a` quick-add, and `H`/`L` moves for the selected task. Review is a read-only filtered queue with selectable rows, inspection detail focus, reason badges, accord/review/state/priority metadata, blockers, and CLI action hints. Logs lists completed work by recency, supports local list/detail browsing, and filters with `/` then Enter/Esc. Rules lists grouped categories and supports add/edit/delete prompts in `src/tui/rules.rs`; Tab has no top-level fallback there. Decisions lists active decisions, supports list/body focus, shows selected metadata/body/path, and supports a basic title/body add prompt in `src/tui/decisions.rs`. Built-in `default-dark` theme styles apply by default, `.tandem/theme.toml` can select the built-in `verdigris` base, and documented color keys can override either palette.
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
