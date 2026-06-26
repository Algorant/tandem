# Tandem Protocol

This directory contains the Tandem protocol planning and, later, protocol implementation assets such as schemas, fixtures, migration notes, and core data-model documentation.

The protocol defines the local-first file format for human/agent coordination.

## Scope

The protocol area owns:

- `.tandem/` workspace layout
- `.tandem/tandem.md` workspace config shape
- active work documents in `.tandem/board/`
- completed work documents in `.tandem/logs/`
- `.tandem/events.jsonl` lifecycle ledger
- `accord` work-agreement model
- review and completion semantics
- Brainfile import/migration compatibility
- schema and fixture definitions once implementation begins

The protocol area does **not** own TUI rendering details. TUI design belongs in `../tandem-tui/`, though both areas must stay synchronized.

## Current status

Planning/specification mode. No protocol crate, schemas, or fixtures exist yet.

## Documentation

- `plan/spec.md` — protocol draft
- `plan/todo.md` — protocol task tracker
- `../README.md` — parent project overview
- `../plan/spec.md` — parent project plan
- `../plan/todo.md` — parent project todo
- `../AGENTS.md` — agent guidance and documentation sync rules

## Sync requirements

This directory must stay aligned with the parent Tandem docs.

When protocol terminology, layout, lifecycle, CLI naming, or scope changes, update all affected docs in the same change:

- `../README.md`
- `../plan/spec.md`
- `../plan/todo.md`
- `README.md`
- `plan/spec.md`
- `plan/todo.md`
- `../AGENTS.md` if agent rules or workflows change

No drift is allowed. If this README contradicts parent docs, fix the contradiction immediately.

## Key current decisions

- Product/protocol name: **Tandem**
- CLI binary: `tdm`
- Protocol data directory: `.tandem/`
- Config file: `.tandem/tandem.md`
- Work agreement object: `accord`
- Completion is an action/archive transition, not a default `done` column.
- Human workflow state, accord state, and review state are separate.
- Logs are first-class completed-work history.
