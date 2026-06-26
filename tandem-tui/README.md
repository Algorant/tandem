# Tandem TUI

This directory contains planning and, later, implementation work for the Tandem terminal UI.

The intended implementation target is Rust + Ratatui.

## Scope

The TUI area owns:

- Ratatui app architecture
- board/review/logs/rules/decisions views
- responsive layouts
- theming
- mouse support and hit-map interaction model
- keyboard and command-palette UX
- review, accord, completion, and logs workflows as presented in the UI
- eventual TUI tests and snapshots

The TUI area does **not** own the underlying protocol semantics. Protocol rules and data-model decisions belong in `../protocol/`, though the TUI must represent them faithfully.

## Current status

Planning/specification mode. No Rust TUI crate exists yet.

## Documentation

- `plan/spec.md` — TUI draft
- `plan/todo.md` — TUI task tracker
- `../README.md` — parent project overview
- `../plan/spec.md` — parent project plan
- `../plan/todo.md` — parent project todo
- `../protocol/README.md` — protocol area overview
- `../protocol/plan/spec.md` — protocol draft the TUI must follow
- `../AGENTS.md` — agent guidance and documentation sync rules

## Sync requirements

This directory must stay aligned with the parent Tandem docs and the protocol docs.

When TUI architecture, invocation, layout, workflow, or status terminology changes, update all affected docs in the same change:

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
- TUI directory: `tandem-tui/`
- CLI binary: `tdm`
- Likely invocation: `tdm tui` initially, with `tdm-tui` possible later.
- TUI implementation target: Rust + Ratatui.
- Do not assume a persistent `done` column.
- Make review, accord status, validation, and logs prominent.
- Theme support is required from the beginning.
- Mouse support should use a hit-map style model.
