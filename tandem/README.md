# Tandem CLI/TUI

This directory contains planning and implementation work for the Tandem user-facing CLI and terminal UI.

Current phase: CLI v0 surface complete for the current known scope, with forward focus on the Rust/Ratatui TUI. `tandem tui` now launches a Board-first shell where delivered work is handled through the Board Validation state/subview rather than a separate top-level Review pane.

## Scope

The CLI/TUI area owns:

- `tandem` CLI command design and user experience
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

Planning/specification plus implementation mode. A Rust binary package now lives in this directory and builds a `tandem` binary with `--version`, `version`, `init`, `list`, `show`, `add`, `move`, `update`, `complete`, `cancel`, `search`, read-only `log`, `accord ready|claim|deliver|accept|rework|block|fail`, `rules list|add|edit|delete`, and `decision list|show|add` coverage. The current known CLI surface is considered complete unless new feature requests or bugs appear. Frontmatter reads use the approved `yaml-rust2` dependency while command mutations use raw-source, minimal-diff patches. Completion writes nested `completion` metadata, accord actions write canonical validation/timestamp metadata, and read paths tolerate earlier flat completion fields. The current `tandem tui` implementation uses Ratatui plus crossterm to render top-level Board, Logs, Rules, and Decisions tabs with built-in `default-dark` and `verdigris` presets, user theme discovery from `~/.config/tandem/themes/*.toml` or `$XDG_CONFIG_HOME/tandem/themes/*.toml`, and workspace theme selection/overrides from `.tandem/theme.toml`. `1`..`4` are the keyboard top-level view switchers, with mouse tab clicks still available as explicit pointer navigation. Board uses Brainfile-style state subviews rather than simultaneous narrow columns: configured states render as count tabs, the active state gets the full Board list area, rows show richer priority/title/accord/review/checklist/tag/assignee/update/path metadata, and selected-item details remain below. Board keeps local state/item navigation, manual `r` reload, idle file-change hot reload with safe warning/error surfacing, keyboard quit, basic mouse wheel/click handling, and first Board mutations: `a` quick-adds a task in the selected/default configured state, while `H`/`L` moves the selected task to the previous/next configured state and reloads the board. Pressing `e` in Board opens the selected active task Markdown document in `$EDITOR`, temporarily restores the terminal while the editor runs, then resumes the TUI and reloads the workspace. Logs stay read-only for generated completed history; decision/custom document editing in `$EDITOR` and raw rules-file editing are deferred while Rules keep their in-TUI prompts. Board Validation surfaces delivered/accepted accord details and A/R/C action hints for approve, request changes, and complete/log CLI flows. Logs has a real completed-work browser: recency-sorted list, selection/navigation, local list/detail focus, detail pane with completion metadata/body/path/event context, safe load warnings, and `/` search filtering across log IDs, titles, summaries, bodies, validation text, and files. Rules lists project rules grouped by `always`/`never`/`prefer`/`context` and supports local category navigation plus add/edit/delete prompts from `src/tui/rules.rs`. Decisions lists active decision documents, supports local list/body focus, shows selected decision metadata/body/path, and supports a basic title/body add prompt from `src/tui/decisions.rs`. `h/j/k/l` stay inside the active view, and Tab/BackTab cycle focus only where a view has meaningful focusable panes. Remaining gaps include additional Board mutations, richer Validation mutation prompts, Decisions references/tags prompt parity, and richer action buttons.

## Build/run

From this directory:

```text
cargo run -- --version
cargo run -- version
cargo run -- init
cargo run -- add --title "Implement next CLI slice"
cargo run -- list
cargo run -- move task-1 --state in-progress
cargo run -- update task-1 --priority high --tag cli
cargo run -- accord ready task-1 --assignee pi --validation "cargo test"
cargo run -- complete task-1 --summary "Implemented and tested"
cargo run -- log list
cargo run -- rules add --category always --rule "Run tests before completing tasks."
cargo run -- tui
```

Use `cargo run -- <command>` during early development. The package binary name is `tandem`.

## Epic, Task, and Subtask hierarchy

The CLI derives roles from resolved documents and enforces one strict three-tier hierarchy:

```text
task-10       Epic: root `type: task`, `kind: epic`; global ID
└── task-11   Task: `parentId: task-10`; global ID; `epic-task`
    └── task-11-1   Subtask: `parentId: task-11`; parent-derived ID; `subtask`
```

Create the same structure with normal `add` commands and always consume the returned IDs:

```text
tandem add --title "Coordinate the release" --kind epic
tandem add --title "Write release notes" --parent task-10
tandem add --title "Check upgrade notes" --parent task-11
```

Epics and Tasks—including standalone, generic-parent, and direct Epic Tasks—allocate the next global `task-N` across active Board documents and completed Logs. Only a Subtask directly beneath a Task allocates the next `<Task ID>-M` suffix. Completed suffixes are not reused. `tandem show` exposes `tasks` for an Epic, `subtasks` for a Task, and no child collection for a Subtask.

Subtasks are leaves. A parented Epic, child beneath a Subtask, direct Epic Task with a hierarchical ID, global-ID Subtask, or deeper task ID is a structural error. There is no legacy compatibility exception. IDs are immutable, so `tandem update --parent` is accepted only when the prospective parent preserves the document's canonical role and existing ID; invalid reparenting is rejected without mutation.

`parentId` may also resolve to a decision or custom document. That child remains a global-ID Task with human label `Parent` and JSON `parentRelationship: "parent"`, and it may own parent-derived Subtasks. Direct Epic Tasks use `Task of Epic` / `epic-task`; only direct Task children use `Subtask of` / `subtask`.

Only Tasks are initial delegation roots. Epics are decomposed into independently delegated Tasks; one delegated Task worker executes its direct Subtasks through a session todo projection and returns one Task-level handoff. Epics and Subtasks are not independently delegated.

The earlier `add --subtask <title>` inline-checklist authoring path is deprecated and rejected. Existing inline `subtasks` metadata remains readable, but the CLI does not create it for new lifecycle-bearing work.

## Release and install target

Current release: `tandem` package version `0.6.0`, `tandem` binary, annotated git tag `tandem-v0.6.0`.

Downstream integrations such as `pi-tandem` can install the released CLI with:

```text
cargo install --git git@github.com:Algorant/tandem.git --tag tandem-v0.6.0 --path tandem --locked
```

For local checkout installs before a tag is published:

```text
cargo install --path tandem --locked
tandem --version
```

`pi-tandem` should locate the installed binary through `TANDEM_BIN` or `tandem` on `$PATH` in that order. Concise public GitHub Release notes are curated in `GITHUB_RELEASE_NOTES.md`; reusable install, validation, known-limitation, and release-command details are in `RELEASE.md`.

## Implemented TUI themes and keys

`tandem tui` starts from the built-in `default-dark` palette, discovers user theme files, applies the user's XDG config, applies any workspace theme override, then applies workspace display config:

1. built-in presets: `default-dark` and `verdigris`
2. user theme files: `$XDG_CONFIG_HOME/tandem/themes/*.toml`, or `~/.config/tandem/themes/*.toml` when `XDG_CONFIG_HOME` is unset
3. user config: `$XDG_CONFIG_HOME/tandem/config.toml`, or `~/.config/tandem/config.toml` when `XDG_CONFIG_HOME` is unset
4. workspace theme selector/override: `.tandem/theme.toml`
5. workspace TUI display config: `.tandem/config.toml`

Use the user config for your normal machine-wide preference:

```toml
theme = "verdigris"
transparent_background = true
badge_style = "muted"
```

Use `.tandem/theme.toml` only when a project should override the user's normal theme. `base`, `builtin`, and `extends` remain accepted selector aliases for existing workspace files. After the selector, user config and workspace `.tandem/theme.toml` may override simple TOML-style string color values (`"#RRGGBB"`, `"#RGB"`, or supported terminal color names) and supported root settings:

```toml
theme = "my-custom-dark"

[colors]
accent = "#8ec07c"
```

Install example presets as user themes with:

```text
mkdir -p ~/.config/tandem/themes
cp tandem/examples/themes/default-dark.toml ~/.config/tandem/themes/default-dark.toml
cp tandem/examples/themes/verdigris.toml ~/.config/tandem/themes/verdigris.toml
```

A user theme file may inherit from a built-in and provide only overrides:

```toml
name = "my-custom-dark"
base = "default-dark"

[colors]
accent = "#8ec07c"
```

Supported built-in presets are `default-dark` (conservative dark/default) and `verdigris` (repo default here). Supported keys:

- root keys: `theme` (workspace selector), `base`, `builtin`, `extends`, `name`, `transparent_background` (optional boolean; default `false`), `badge_style` (optional; default `muted`) or `[badges] style`
- `[colors]`: `background`, `panel`, `text`, `muted`, `accent`, `success`, `warning`, `error`, `border`, `selected_bg`, `selected_fg`
- `[priority]`: `critical`, `high`, `medium`, `low`, `none`
- `[badges]`: `style` for visual badge styling; legacy `disabled` remains accepted during migration
- `[board.badges]` or legacy `[badges]`: `disabled` (list of built-in badge IDs or configured tag names to suppress)
- `[board.badges.tags.<tag>]` or legacy `[badges.tags.<tag>]`: optional `label` and optional `tone` (`accent`, `success`, `warning`, `error`, or `muted`) for project/user opt-in tag badges
- `[badges.accord]`: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`, `unknown`
- `[badges.review]`: `not-ready`, `pending`, `accepted`, `changes-requested`, `rejected`, `failed`, `unknown`

In the TUI, use `1`..`4` to switch Board/Logs/Rules/Decisions, arrow keys or `j`/`k` to move, `e` in Board to edit the selected active task in `$EDITOR`, `/` in Logs to filter, `?` for help, and `q` to quit. Mouse clicks use a hit-map: top tabs switch views, Board state tabs switch state subviews, Board/Logs rows select items, clicking an already-selected Board row toggles its inline preview, footer command labels run the same keyboard actions where safe, and non-action regions are safe no-ops. `h/l` stays local: Board state subviews, Logs/Decisions list-detail focus, and Rules categories. Tab/BackTab cycles focus only in views with focusable panes; in Rules it stays in view and shows a hint instead of switching top-level views. A manual PTY smoke should confirm the status line includes `theme built-in verdigris + .../.config/tandem/config.toml` for global selection, or a `.tandem/theme.toml` suffix when a workspace overrides it. Invalid user/workspace theme or display config files are non-fatal and appear as warnings in the status line. Remove the user or workspace selector to return to `default-dark`.

`transparent_background = true` may be set in a user theme, user config, or `.tandem/theme.toml` to avoid forcing the app/panel background colors and let terminal default or compositor transparency show through where practical. Omitted themes keep the current opaque background behavior.

`badge_style` controls Board priority/status/tag chips: `muted` (default softened fill), `accent` (small colored rail), `text` (colored label), `ghost` (transparent chip outline), or `solid` (legacy saturated fill). Rounded-edge badge rendering remains deferred.

Default Board badges are limited to priority (`CRIT`, `HIGH`, `MED`, `LOW`), work-type tags (`RESEARCH`, `SPIKE`, `DELIVERABLE`), validation `VISUAL`, attention accord/review statuses, and subtask progress. Project/domain tags such as `tui`, `cli`, `docs`, `spec`, or `protocol` are opt-in, not global defaults. Configure extra tag badges or suppress badges in user config or workspace `.tandem/config.toml`:

```toml
[board.badges]
disabled = ["deliverable", "visual"]

[board.badges.tags.tui]
label = "TUI"
tone = "accent"

[board.badges.tags.docs]
# label defaults to "DOCS"
tone = "success"
```

User config applies before workspace `.tandem/config.toml`; workspace badge config is therefore the final override. Legacy `[badges]` / `[badges.tags.*]` sections in theme files are still read during migration, but `.tandem/theme.toml` should remain for theme selection/color/style behavior. `disabled` is a simple suppression list for built-ins or configured tag names, not a regex/rule engine.

`NO_COLOR=1` or `TANDEM_NO_COLOR=1` uses the terminal/no-color fallback even when user/workspace config selects Verdigris or a user theme.

## Documentation

- `plan/spec.md` — CLI/TUI draft
- `plan/todo.md` — CLI/TUI task tracker
- `../README.md` — parent project overview
- `../plan/spec.md` — parent project plan
- `../plan/todo.md` — parent project todo
- `../protocol/README.md` — protocol area overview
- `../protocol/plan/spec.md` — protocol draft the CLI/TUI must follow
- `../AGENTS.md` — agent guidance

## Key current decisions

- Product/protocol name: **Tandem**
- CLI/TUI directory: `tandem/`
- CLI binary: `tandem`
- CLI design and the current known CLI v0 implementation came before TUI implementation; future CLI work should be explicit new features or bug fixes.
- V0 TUI invocation: `tandem tui` only.
- TUI implementation target: Rust + Ratatui with crossterm terminal events/backend.
- `tandem tui` currently has top-level Board, Logs, Rules, and Decisions tabs. `1`..`4` are the keyboard view switchers, mouse tab clicks switch views explicitly, and local navigation keys stay inside the active view. Board has state subview tabs with counts, a full-width selected-state list with richer rows, `a` quick-add, `H`/`L` moves for the selected task, and `e` opens the selected active task document in `$EDITOR`. Logs lists completed work by recency, supports local list/detail browsing, filters with `/` then Enter/Esc, and remains read-only for editor-open behavior. Rules lists grouped categories and supports add/edit/delete prompts in `src/tui/rules.rs`; Tab has no top-level fallback there. Decisions lists active ADR-compatible `decision` records, supports list/body focus, shows selected metadata/body/path, and supports a basic title/body add prompt in `src/tui/decisions.rs`; `$EDITOR` editing for decisions/custom documents is deferred. Built-in `default-dark` and `verdigris` presets apply by default/selection, user themes are discovered from `~/.config/tandem/themes/*.toml` or `$XDG_CONFIG_HOME/tandem/themes/*.toml`, `.tandem/theme.toml` can select a built-in or user theme, and documented color keys can override the selected palette.
- Basic feature parity with live Brainfile CLI/TUI is the baseline; improvements and omissions must be intentional.
- Do not assume a persistent `done` column.
- Make review, accord status, validation, and logs prominent.
- Theme support is required from the beginning.
- Mouse support should use a hit-map style model, be enabled by default, and exclude drag/drop in v0.


## Locked v0 CLI/TUI decisions

- v0 commands: `--version`, `version`, `init`, `list`, `show`, `add`, `move`, `update`, `complete`, `cancel`, `log`, `search`, `accord`, `rules`, `decision`, `tui`.
- `tandem log`: `list`, `show`, `search`.
- `tandem rules`: `list`, `add`, `edit`, `delete`.
- `tandem accord`: `ready`, `claim`, `deliver`, `accept`, `rework`, `block`, `fail`.
- Human-readable output by default: compact tables for list/search and labeled detail blocks for show/log/decision.
- All read commands support `--json` using `{ "ok": true, "data": ..., "warnings": [] }` envelopes.
- V0 CLI uses canonical command names and long flags only; no short aliases.
- First implementation language: Rust inside `tandem/`.
- `tandem decision`: `list`, `show`, `add`.
- First TUI MVP: board mutations immediately; Board, Logs, Rules, Decisions views; Board Validation workflow, theme, and mouse support included.
- Validation queue: Board state/subview for delivered work awaiting accept/rework/complete in v0.
- Keymaps: fixed defaults in v0; custom keymap config later.
- Markdown rendering: styled basics in v0.
- Theme config loading order: built-in defaults, then user TOML themes in `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, then user config in `$XDG_CONFIG_HOME/tandem/config.toml` or `~/.config/tandem/config.toml`, then workspace selector/override at `.tandem/theme.toml`; Board display settings such as project tag badges load from user config and workspace `.tandem/config.toml`.
- Deferred from v0: non-core command families and integrations listed in `plan/spec.md`, plus schemas, fixtures, and root Rust workspace layout.
