# Tandem CLI/TUI Todo

Status: active TUI implementation
Last updated: 2026-06-27

This todo tracks CLI/TUI planning and implementation tasks. The current CLI/TUI draft lives in `tandem/plan/spec.md`.

## Accomplished

- [x] Chose CLI-first sequencing: design `tandem` before the interactive TUI.
- [x] Chose first implementation language: Rust inside `tandem/`.
- [x] Kept implementation layout and dependency choices open until implementation starts.
- [x] Locked v0 CLI command families:
  - `init`
  - `list`
  - `show`
  - `add`
  - `move`
  - `complete`
  - `log`
  - `search`
  - `accord`
  - `rules`
  - `decision`
  - `tui`
- [x] Locked v0 `log` scope: `list`, `show`, `search`.
- [x] Locked v0 `rules` scope: `list`, `add`, `edit`, `delete`.
- [x] Locked v0 `accord` scope: `ready`, `claim`, `deliver`, `accept`, `rework`, `block`, `fail`.
- [x] Locked CLI output direction: human-readable by default, with `--json` for all read commands.
- [x] Locked v0 command-name policy: canonical commands and long flags only; no abbreviated flags or alias commands.
- [x] Locked human-readable output style: compact tables for list/search, labeled detail blocks for show/log/decision.
- [x] Locked `--json` response envelope: `{ "ok": true, "data": ..., "warnings": [] }`.
- [x] Locked `tandem decision` v0 scope: `list`, `show`, `add`.
- [x] Locked v0 TUI invocation: `tandem tui` only.
- [x] Deferred non-v0 command families and project structure: template features, schema-management commands, assistant integration commands, credential/provider commands, third-party archive/export integrations, schemas, fixtures, and root Rust workspace layout.
- [x] Locked first TUI MVP views:
  - Board
  - Review
  - Logs
  - Rules
  - Decisions
- [x] Locked first TUI MVP inclusion of board mutations.
- [x] Locked theme support into the first TUI MVP.
- [x] Locked theme config loading order: built-in defaults, then user config, then workspace config.
- [x] Locked v0 theme file policy: TOML user themes in `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml` and workspace selector/override at `.tandem/theme.toml`.
- [x] Locked mouse support into the first TUI MVP, enabled by default for click/scroll/tab/action-button interactions.
- [x] Excluded drag/drop from v0.
- [x] Locked fixed default keybindings for v0; keymap config is deferred.
- [x] Locked styled-basic Markdown rendering for v0.
- [x] Superseded the simple filtered-list Review queue direction by folding delivered-work handling into Board Validation.
- [x] Captured current Brainfile TUI issues to avoid:
  - progress tied to a persistent completion state
  - hardcoded theming
  - missing mouse support
  - weak logs/review surfaces
- [x] Drafted responsive layout modes:
  - wide
  - medium
  - narrow
  - tiny-terminal fallback
- [x] Drafted keyboard model and command palette ideas.
- [x] Drafted review, completion, logs, and accord UX.
- [x] Added `tandem/README.md` for CLI/TUI-area documentation.
- [x] Drafted implementation-facing `tandem` v0 command reference covering every locked command family.
- [x] Defined command purpose, canonical long-flag syntax, inputs, output shape, command kind, and obvious exit/error notes.
- [x] Defined `--json` data payload examples for `list`, `show`, `search`, `log list`, `log show`, `log search`, `rules list`, `decision list`, and `decision show`.
- [x] Added examples for completion warnings and accord/rules/decision mutations.
- [x] Defined `tandem decision list|show|add` v0 command model.
- [x] Defined v0 `tandem log` output fields for list, show, and search.
- [x] Defined v0 rules CLI add/edit/delete flow.
- [x] Preserved review metadata sort/detail lessons for Board Validation and logs rather than a top-level Review view.
- [x] Planned minimal-diff write behavior for CLI/TUI mutations, including raw source preservation, atomic writes, concurrent edit detection, timestamp discipline, and separate event appends.
- [x] Started the Rust implementation inside `tandem/` with a single `tandem` binary package.
- [x] Implemented the first CLI slice:
  - `tandem init [--title <title>]`
  - `tandem list [--state <state>] [--type <type>] [--json]`
  - `tandem show <id> [--json]`
  - `tandem tui` stub message
- [x] Implemented the next useful CLI slice:
  - `tandem add --title <title> ...`
  - `tandem move <id> --state <state>`
  - `tandem complete <id> --summary <text> ...`
  - `tandem search <query> [--state <state>] [--type <type>] [--json]`
  - `tandem log list|show|search`
  - `tandem rules list`
  - `tandem decision list|show|add`
- [x] Defined implemented exit-code categories in code and CLI docs: success `0`, runtime/data/write failures `1`, usage/argument failures `2`.
- [x] Defined implemented empty/no-match read behavior: human read commands print an explicit empty message, while JSON read commands return empty arrays/counts and exit `0`.
- [x] Added atomic document writes, lifecycle event appends, UTC timestamps, and simple file-change checks for `move` and `complete`.
- [x] Integrated the approved `yaml-rust2` dependency for frontmatter/config/document read parsing while preserving raw body/source mutation behavior.
- [x] Added targeted parser tests for nested accord/review statuses, inline and block arrays, structured rules, and scalar behavior.
- [x] Implemented `tandem rules add|edit|delete` with raw-source config patching and `rules.updated` event appends.
- [x] Implemented `tandem accord ready|claim|deliver|accept|rework|block|fail` with nested accord frontmatter patching and `accord.*` event appends.
- [x] Tightened completion metadata writes to use nested `completion` frontmatter while preserving read compatibility with earlier flat completion fields.
- [x] Tightened accord metadata writes to include claim/delivery timestamps and canonical `accord.validation.commands` while reading earlier `accord.validations` fields.
- [x] Added structural mutation validation for active task moves, completion, and accord actions, including canonical accord/review status checks.
- [x] Defined clearer CLI error message categories for parse failures, validation failures, write conflicts/write failures, and event append failures.
- [x] Added unit coverage for nested completion metadata, legacy completion reads, canonical accord validation metadata, and invalid review-status validation.
- [x] Considered the current known v0 CLI surface complete; future CLI work should be explicit new features or bug fixes.
- [x] Added the minimal Ratatui/crossterm dependency stack for `tandem tui` without adding theme/TOML/Markdown parser dependencies.
- [x] Replaced the `tandem tui` stub with a read-only Board-first TUI shell in `src/tui.rs`.
- [x] Implemented the initial direct crossterm event loop with alternate-screen setup, raw mode, mouse capture, reload, help, and safe cleanup on quit.
- [x] Rendered active `.tandem/board` documents by configured state, including an `unfiled` bucket for state-less active documents.
- [x] Added keyboard navigation across states/items, selected-item detail scrolling, basic mouse click/wheel handling, and unit coverage for state bucket behavior.
- [x] Implemented the first in-TUI Board mutation: `H`/`L` moves the selected task to the previous/next configured state, reloads after mutation, and surfaces move errors in the status line.
- [x] Implemented TUI quick-add: `a` opens a title prompt, Enter creates a basic task in the selected/default configured state, Esc cancels, and success reloads/selects the new task.
- [x] Implemented top-level TUI view switching: Board, Logs, Rules, and Decisions tabs; `1`..`4` keyboard switching; mouse tab switching; and initial non-Board view scaffolding while preserving Board quick-add and move flows. Validation now lives as a Board state/subview.
- [x] Folded the prior read-only Review queue direction into Board Validation:
  - filters active items needing attention: delivered accords, pending/in-review items, changes-requested/rejected/failed reviews, blocked/failed/rework accords, accepted active accords, validation failures, and blockers
  - sorts priority first, then most recently delivered/updated
  - renders selectable queue rows plus inspection detail with reason badges/lines, accord/review/state/priority metadata, blockers, delivered evidence/files, and CLI action hints
- [x] Implemented useful Rules and Decisions TUI views:
  - Rules lists `always`/`never`/`prefer`/`context` groups with selection plus add/edit/delete prompts.
  - Decisions lists active decision docs, shows selected metadata/body/path, and adds basic title/body decisions.
- [x] Split Rules and Decisions TUI view code into dedicated `src/tui/rules.rs` and `src/tui/decisions.rs` modules.
- [x] Implemented the first TUI theme foundation without new dependencies:
  - built-in `default-dark` semantic palette
  - workspace `.tandem/theme.toml` overrides for `[colors]`, `[priority]`, `[badges.accord]`, and `[badges.review]`
  - status-line warnings for unknown keys or invalid colors
  - `NO_COLOR`/`TANDEM_NO_COLOR` terminal fallback
  - Board styling for headers, tabs, borders, selection, priority, accord, review, details, and status lines
- [x] Added a selectable built-in `verdigris` TUI theme, root `base`/`builtin`/`extends` theme selection keys, and `tandem/examples/themes/verdigris.toml` for immediate visual evaluation.
- [x] Implemented the TUI Logs browser:
  - recency-sorted completed-log list from `.tandem/logs/`
  - selected-log detail pane with completion summary, completed timestamp, files changed, validation, reviewer, accord/review status, accord detail/evidence, body, path, and event context
  - `/` search prompt with Enter apply and Esc cancel/clear
  - empty/no-match states and safe per-log/event load warnings
- [x] Reworked the Board from simultaneous columns into Brainfile-style state subviews:
  - configured states render as count-labeled tabs with mouse selection
  - the active state gets the full Board list area
  - rows include richer priority, title, accord/review, checklist, tag, assignee, updated, file-count, path, and right-ish ID metadata
  - quick-add, `H`/`L` task moves, detail focus, theme styling, and mouse wheel/click behavior remain active
- [x] Added first-class user theme discovery and preset examples:
  - discovers `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`
  - lets `.tandem/theme.toml` select built-in or user themes with `theme = "name"` before applying workspace overrides
  - keeps invalid theme files non-fatal with status warnings
  - documents `default-dark` and `verdigris` examples under `tandem/examples/themes/`
- [x] Tightened default TUI keyboard/focus semantics:
  - `1`..`5` are the keyboard top-level view switchers; local keys no longer switch top-level views.
  - `h/j/k/l` stay inside the active view: Board state/items, Logs/Decisions list-detail focus or scroll, and Rules category/list movement.
  - Tab/BackTab cycle focus only in views with meaningful focusable panes and show a Rules hint instead of falling back to top-level view cycling.
  - Added unit coverage for numeric switching, local `h/j/k/l`, and Tab no-fallback behavior.
  - Manual PTY smoke (2026-06-28): `target/debug/tandem tui` accepted scripted `1`..`5`, local `h/j/k/l`, and Tab/BackTab sequences and exited cleanly (`rc=0`).

## Current tasks

- [x] Define numeric exit-code categories for CLI implementation.
- [x] Define exact no-match and empty-list behavior for implemented read commands.
- [x] Replace first-slice YAML-ish frontmatter parsing with a more complete parser while preserving minimal-diff behavior.
- [x] Implement `tandem add`, `tandem move`, and `tandem complete`.
- [x] Implement `tandem search`, `tandem log list|show|search`, `tandem rules list`, and `tandem decision list|show` read commands.
- [x] Implement `tandem accord ready|claim|deliver|accept|rework|block|fail`.
- [x] Implement `tandem rules add|edit|delete`.
- [x] Define user-facing messages for write conflicts, parse failures, validation failures, and event append failures.
- [x] Define accord badge/status visual language for the current Board shell.
- [x] Specify and implement the default/configurable Board badge set, including `DELIVERABLE`, opt-in tag badges, optional label/tone, and simple badge suppression config.
- [x] Define minimal implementation layout inside `tandem/` and dependency choices only when implementation begins.
- [x] Decide initial Ratatui event loop approach.
- [x] Define exact TOML theme keys for workspace override in the current theme foundation.
- [x] Add full user theme discovery from `~/.config/tandem/themes/*.toml` and `$XDG_CONFIG_HOME/tandem/themes/*.toml`.
- [ ] Define final fixed keyboard default table for v0.
- [ ] Define styled-basic Markdown rendering details.

## Next recommended steps

1. Add safe Review action buttons/mutations on top of the current read-only queue, likely accord accept/rework and completion/archive prompts.
2. Continue Board mutations after quick-add and move/change-state, likely edit, complete, or accord actions.
3. Extend Decisions with references/tags prompts only if the TUI needs full CLI option parity beyond the basic title/body flow.
4. Add any additional built-in palettes beyond Verdigris only as explicit theme requests; next broad polish remains final mouse hit-map, keyboard, and styled-basic Markdown requirements at MVP level.
5. Change existing CLI behavior only for explicit new feature requests or bug fixes.

## First TUI MVP checklist

- [x] Render Board, Logs, Rules, and Decisions views at shell/placeholder level.
- [x] Render Validation as the Board state/subview for delivered work with inspection detail and action hints.
- [x] Navigate state subviews/items and view details.
- [x] Add items from the Board view.
- [x] Move items between states.
- [ ] Edit items from the TUI.
- [ ] Complete items to logs.
- [ ] Run accord actions from detail/review flows.
- [x] Add/edit/delete rules.
- [x] Browse active decisions and add basic title/body decisions.
- [x] Show and search logs.
- [x] Load and apply built-in, user-discovered, plus workspace selector/override themes.
- [x] Support mouse selection, scrolling, and tab/subview switching by default.
- [ ] Add action-button click interactions.
- [ ] Confirm drag/drop is absent from v0 interactions.
- [ ] Watch/reload file changes.
- [ ] Surface parse and write errors safely.

## Acceptance criteria for first usable TUI

- [x] Does not assume a persistent completion state.
- [x] Makes the Board Validation flow obvious.
- [x] Makes accord state obvious at a basic status-badge level.
- [x] Supports board mutations immediately.
- [x] Supports built-in, user-discovered, plus workspace selector/override themes.
- [x] Supports mouse selection and scroll.
- [ ] Handles external file edits without crashing.
- [x] Keeps logs useful, searchable, and inspectable.
