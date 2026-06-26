# Tandem

Tandem is a draft local-first protocol and toolchain for human/agent project coordination.

It takes inspiration from Brainfile's file-based task board model, but leans harder into collaborative workflows: humans and agents agree on work through **accords**, move delivered work through review, and preserve completed work in useful logs.

## Monorepo layout

```text
plan/          Parent project planning and cross-cutting todos
protocol/      Tandem protocol/specification work
tandem-tui/    Rust/Ratatui TUI planning and implementation
```

## Naming

- Product/protocol: **Tandem**
- Repository: `tandem`
- Project data directory: `.tandem/`
- Project config file: `.tandem/tandem.md`
- CLI binary: `tdm`
- TUI directory: `tandem-tui/`
- Work agreement object: `accord`
- Future prefixes: `td` / `tdm`

## Documentation contract

Every discrete aspect of Tandem must have its own documentation and keep it synchronized with the parent project docs. No documentation drift is allowed.

Minimum documentation set for each major area:

- `README.md` — purpose, scope, layout, current status, and links.
- `plan/spec.md` — design/specification.
- `plan/todo.md` — accomplished/current/next work tracking.

When naming, scope, architecture, lifecycle, or workflow decisions change, update the parent docs and all affected sub-area docs in the same change.

## Current docs

- `AGENTS.md` — guidance for AI agents working in this repo
- `plan/spec.md` — parent project plan
- `plan/todo.md` — parent project todo
- `protocol/README.md` — protocol area README
- `protocol/plan/spec.md` — protocol draft
- `protocol/plan/todo.md` — protocol todo
- `tandem-tui/README.md` — TUI area README
- `tandem-tui/plan/spec.md` — TUI draft
- `tandem-tui/plan/todo.md` — TUI todo
