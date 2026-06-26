# Tandem TUI Todo

Status: active planning  
Last updated: 2026-06-26

This todo tracks TUI-specific work. The current TUI draft lives in `tandem-tui/plan/spec.md`.

## Accomplished

- [x] Chose implementation direction: Rust + Ratatui.
- [x] Captured current Brainfile TUI issues to avoid:
  - fake progress tied to a `done` column
  - hardcoded theming
  - missing mouse support
  - weak logs/review surfaces
- [x] Defined desired core views:
  - Board
  - Detail
  - Review queue
  - Logs
  - Rules
  - Decisions/notes
- [x] Captured responsive layout modes:
  - wide
  - medium
  - narrow
  - tiny-terminal fallback
- [x] Drafted theming requirements and example `theme.toml`.
- [x] Drafted mouse hit-map architecture.
- [x] Drafted keyboard model and command palette ideas.
- [x] Drafted review, completion, logs, and accord UX.
- [x] Drafted MVP phases.
- [x] Added `tandem-tui/README.md` for TUI-area documentation.

## Current tasks

- [ ] Keep `tandem-tui/README.md`, `plan/spec.md`, and `plan/todo.md` synchronized with parent and protocol docs.
- [ ] Decide final crate/package layout for the TUI.
- [ ] Decide whether the TUI lives as:
  - `tdm tui`
  - `tdm-tui`
  - both
- [ ] Decide initial Ratatui event loop approach.
- [ ] Decide theme file format and loading order.
- [ ] Decide initial keyboard defaults and keymap customization format.
- [ ] Decide whether mouse mode is enabled by default.
- [ ] Decide if drag/drop is MVP or post-MVP.
- [ ] Decide how much Markdown rendering is needed for v0.
- [ ] Define visual language for accord/review/status badges.

## Next recommended steps

1. Create static mock fixtures from the protocol examples.
2. Build a read-only Ratatui board using test data.
3. Add layout breakpoint tests with `ratatui::backend::TestBackend`.
4. Implement theme structs and built-in themes.
5. Implement keyboard action mapping.
6. Implement mouse hit-map selection and scroll.
7. Wire to `tandem-core` once protocol parsing exists.
8. Update parent and protocol docs whenever TUI decisions affect naming, invocation, or protocol-facing workflow.
9. Add mutation flows:
   - move state
   - update accord
   - review accept/request changes
   - complete/archive
   - restore logs

## MVP checklist

### Phase 1: read-only prototype

- [ ] Render board from fixture data.
- [ ] Navigate states/items.
- [ ] Show detail pane/expanded card.
- [ ] Load built-in theme.
- [ ] Render logs fixture.

### Phase 2: real files

- [ ] Discover `.tandem/tandem.md`.
- [ ] Read active documents.
- [ ] Read logs.
- [ ] Watch for file changes.
- [ ] Surface parse errors safely.

### Phase 3: workflows

- [ ] Add/move work.
- [ ] Edit in `$EDITOR`.
- [ ] Claim/deliver/accept accord.
- [ ] Request/accept review.
- [ ] Complete/archive to logs.
- [ ] Restore/reopen from logs.

## Acceptance criteria for first usable TUI

- [ ] Does not assume a `done` column.
- [ ] Makes review queue obvious.
- [ ] Makes accord state obvious.
- [ ] Supports themes.
- [ ] Supports mouse selection and scroll.
- [ ] Handles external file edits without crashing.
- [ ] Keeps logs useful, searchable, and restorable.
