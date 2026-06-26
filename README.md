# Tandem

Tandem is a draft local-first protocol and toolchain for human/agent project coordination.

It takes inspiration from Brainfile's file-based task board model, but leans harder into collaborative workflows: humans and agents agree on work through **accords**, move delivered work through review, and preserve completed work in useful logs.

Current design baseline: use Brainfile as inspiration for the general protocol/CLI/TUI shape, adapt it into Tandem terminology, and fold in the local Brainfile v3 direction around review, completion/archive, and first-class logs. Tandem does not require Brainfile import/migration or ongoing Brainfile nomenclature compatibility. CLI/TUI work is still in planning: decide the CLI first, then the TUI.

## Monorepo layout

```text
plan/          Parent project planning and cross-cutting todos
protocol/      Tandem protocol/specification work
tandem-tui/    CLI + Rust/Ratatui TUI planning and implementation
```

## Naming

- Product/protocol: **Tandem**
- Repository: `tandem`
- Project data directory: `.tandem/`
- Project config file: `.tandem/tandem.md`
- CLI binary: `tdm`
- CLI/TUI directory: `tandem-tui/`
- Work agreement object: `accord`
- Future prefixes: `td` / `tdm`

## Locked v0 scope

- Protocol fields: `state`/`states`, `type: task`, sequential `task-N` IDs.
- Default states: `todo`, `in-progress`, `review`.
- Document types: `task` and `decision`; custom types are config-only.
- Accord statuses: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`.
- Logs: archived markdown docs in `.tandem/logs/`; lifecycle events in `.tandem/events.jsonl`.
- v0 `tdm` commands: `init`, `list`, `show`, `add`, `move`, `complete`, `log`, `search`, `accord`, `rules`, `decision`, `tui`.
- First implementation: Rust inside `tandem-tui/`.
- First TUI MVP: board mutations, Board/Review/Logs/Rules/Decisions views, theme support, and mouse support.

## Documentation contract

Every discrete aspect of Tandem must have its own documentation and keep it synchronized with the parent project docs. No documentation drift is allowed.

Minimum documentation set for each major area:

- `README.md` — purpose, scope, layout, current status, and links.
- `plan/spec.md` — design/specification.
- `plan/todo.md` — accomplished/current/next work tracking.

When naming, scope, architecture, lifecycle, or workflow decisions change, update the parent docs and all affected sub-area docs in the same change. Keep the project simple: do not treat Rust workspace layout, crates, dependency choices, or schema/fixture directories as settled until explicitly approved.

## Current docs

- `AGENTS.md` — guidance for AI agents working in this repo
- `plan/spec.md` — parent project plan
- `plan/todo.md` — parent project todo
- `protocol/README.md` — protocol area README
- `protocol/plan/spec.md` — protocol draft
- `protocol/plan/todo.md` — protocol todo
- `tandem-tui/README.md` — CLI/TUI area README
- `tandem-tui/plan/spec.md` — CLI/TUI draft
- `tandem-tui/plan/todo.md` — CLI/TUI todo
