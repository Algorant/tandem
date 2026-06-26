# Tandem CLI/TUI Todo

Status: active planning  
Last updated: 2026-06-26

This todo tracks CLI/TUI planning tasks. The current CLI/TUI draft lives in `tandem-tui/plan/spec.md`.

## Accomplished

- [x] Chose CLI-first sequencing: design `tdm` before the interactive TUI.
- [x] Chose first implementation language: Rust inside `tandem-tui/`.
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
- [x] Locked `tdm decision` v0 scope: `list`, `show`, `add`.
- [x] Locked v0 TUI invocation: `tdm tui` only.
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
- [x] Locked v0 theme file policy: TOML user themes in `~/.config/tandem/themes/*.toml` and workspace override at `.tandem/theme.toml`.
- [x] Locked mouse support into the first TUI MVP, enabled by default for click/scroll/tab/action-button interactions.
- [x] Excluded drag/drop from v0.
- [x] Locked fixed default keybindings for v0; keymap config is deferred.
- [x] Locked styled-basic Markdown rendering for v0.
- [x] Chose a simple filtered-list Review queue for v0 instead of opinionated hard-coded sections.
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
- [x] Added `tandem-tui/README.md` for CLI/TUI-area documentation.
- [x] Drafted implementation-facing `tdm` v0 command reference covering every locked command family.
- [x] Defined command purpose, canonical long-flag syntax, inputs, output shape, command kind, and obvious exit/error notes.
- [x] Defined `--json` data payload examples for `list`, `show`, `search`, `log list`, `log show`, `log search`, `rules list`, `decision list`, and `decision show`.
- [x] Added examples for completion warnings and accord/rules/decision mutations.
- [x] Defined `tdm decision list|show|add` v0 command model.
- [x] Defined v0 `tdm log` output fields for list, show, and search.
- [x] Defined v0 rules CLI add/edit/delete flow.
- [x] Defined simple Review view sort direction: priority first, then recently updated or delivered.
- [x] Planned minimal-diff write behavior for CLI/TUI mutations, including raw source preservation, atomic writes, concurrent edit detection, timestamp discipline, and separate event appends.

## Current tasks

- [ ] Keep `tandem-tui/README.md`, `plan/spec.md`, and `plan/todo.md` synchronized with parent and protocol docs.
- [ ] Define numeric exit-code categories for CLI implementation.
- [ ] Define exact no-match and empty-list behavior for each read command.
- [ ] Define user-facing messages for write conflicts, parse failures, validation failures, and event append failures.
- [ ] Define accord badge/status visual language.
- [ ] Define minimal implementation layout inside `tandem-tui/` and dependency choices only when implementation begins.
- [ ] Decide initial Ratatui event loop approach.
- [ ] Define exact TOML theme keys for user themes and workspace override.
- [ ] Define final fixed keyboard default table for v0.
- [ ] Define styled-basic Markdown rendering details.

## Next recommended steps

1. Review the command reference with the orchestrator and adjust any command options or output shapes.
2. Define numeric CLI exit-code categories, empty/no-match read-command behavior, and write-failure messages.
3. Convert minimal-diff write behavior into an implementation checklist when coding begins.
4. Draft the first TUI MVP interaction flows for Board, Review, Logs, Rules, and Decisions.
5. Draft final theme, mouse, keyboard, and styled-basic Markdown requirements at MVP level.
6. Update parent/protocol docs only if any CLI/TUI decision changes protocol-facing behavior.
7. Start implementation planning only after command and TUI behavior are accepted.

## First TUI MVP checklist

- [ ] Render Board, Review, Logs, Rules, and Decisions views.
- [ ] Navigate states/items and view details.
- [ ] Add items from the Board view.
- [ ] Move items between states.
- [ ] Edit items from the TUI.
- [ ] Complete items to logs.
- [ ] Run accord actions from detail/review flows.
- [ ] Add/edit/delete rules.
- [ ] Show and search logs.
- [ ] Load and apply themes.
- [ ] Support mouse selection, scrolling, tab switching, and action buttons by default.
- [ ] Confirm drag/drop is absent from v0 interactions.
- [ ] Watch/reload file changes.
- [ ] Surface parse and write errors safely.

## Acceptance criteria for first usable TUI

- [ ] Does not assume a persistent completion state.
- [ ] Makes the simple filtered Review queue obvious.
- [ ] Makes accord state obvious.
- [ ] Supports board mutations immediately.
- [ ] Supports themes.
- [ ] Supports mouse selection and scroll.
- [ ] Handles external file edits without crashing.
- [ ] Keeps logs useful, searchable, and inspectable.
