# Tandem

Tandem is a draft local-first protocol and toolchain for human/agent project coordination.

It takes inspiration from Brainfile's file-based task board model, but leans harder into collaborative workflows: humans and agents agree on work through **accords**, move delivered work through validation, and preserve completed work in useful logs.

Current design baseline: use Brainfile as inspiration for the general protocol/CLI/TUI shape, adapt it into Tandem terminology, and fold in the local Brainfile v3 direction around validation/review metadata, completion/archive, and first-class logs. Tandem does not require Brainfile import/migration or ongoing Brainfile nomenclature compatibility. The v0 `tandem` CLI surface is implemented and considered complete for the current known scope; forward implementation focus is the Rust/Ratatui TUI plus lightweight agent/editor integrations over `tandem`.

## Monorepo layout

```text
plan/          Parent project planning and cross-cutting todos
protocol/      Tandem protocol/specification work
tandem/    CLI + Rust/Ratatui TUI planning and implementation
extensions/    Agent/editor integrations such as the pi-tandem adapter
```

## Naming

- Product/protocol: **Tandem**
- Repository: `tandem`
- Project data directory: `.tandem/`
- Project config file: `.tandem/tandem.md`
- CLI binary: `tandem`
- CLI/TUI directory: `tandem/`
- Integrations directory: `extensions/`
- Pi extension adapter: `pi-tandem`
- Work agreement object: `accord`
- User-facing CLI: `tandem`; reserved future/internal prefix: `td`

## Locked v0 scope

- Protocol version: `0.1.0` for the first v0 draft.
- Protocol fields: `state`/`states`, `type: task`, sequential `task-N` IDs.
- Default states: `todo`, `in-progress`, `validation` (existing `state: review` files are legacy-compatible reads).
- Document types: `task` and `decision`; custom types are config-only.
- Epics: convention-only `type: task` documents with `kind: epic`; child tasks use `parentId`, loose context uses `references`, and v0 has no separate epic type, ID namespace, or lifecycle.
- Accord statuses: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`.
- Logs: archived markdown docs in `.tandem/logs/`; minimal audit-only lifecycle events in per-actor `.tandem/events/<actor_id>.jsonl` logs, with legacy `.tandem/events.jsonl` still readable during transition.
- v0 `tandem` commands: `init`, `list`, `show`, `add`, `move`, `complete`, `log`, `search`, `accord`, `rules`, `decision`, `tui`; `tandem decision` supports `list`, `show`, `add`.
- Validation: strict structure/core refs; unresolved `parentId`/`blockers` are errors while related `references` are warnings.
- Decision docs: ADR-compatible durable records using `type: decision`; no separate ADR type or lifecycle field required in v0.
- First implementation: Rust inside `tandem/`, currently as one `tandem` binary crate with `yaml-rust2` parsing, raw-source CLI mutations, and a Ratatui/crossterm TUI module.
- CLI output: human-readable by default using compact tables/detail blocks; all read commands support `--json` envelope objects.
- TUI invocation: `tandem tui` only in v0.
- First TUI MVP: board mutations, Board/Logs/Rules/Decisions views plus Board Validation flow, theme support, mouse enabled by default without drag/drop, fixed default keymaps, and styled-basic Markdown rendering; the current Board uses count-labeled state subviews with a full-width selected-state list rather than simultaneous columns.
- V0 CLI aliases: none; canonical commands and long flags only.
- V0 repo shape: CLI/TUI implementation stays under `tandem/`; agent/editor adapters live under `extensions/`; no root Rust workspace, schemas, or fixtures.
- Theme config loading order: built-in defaults, user TOML themes in `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, user config in `$XDG_CONFIG_HOME/tandem/config.toml` or `~/.config/tandem/config.toml`, then workspace selector/override at `.tandem/theme.toml`.
- Planning docs remain Markdown for now; migrate/dogfood Tandem documents after the TUI can manage them safely.
- `extensions/pi-tandem` is a lightweight Pi adapter over an installed `tandem` CLI. It uses safe argument arrays, prefers `tandem --json` read paths, and does not duplicate Tandem protocol parsing/mutation logic.

## Current docs

- `AGENTS.md` — guidance for AI agents working in this repo
- `plan/spec.md` — parent project plan
- `plan/todo.md` — parent project todo
- `protocol/README.md` — protocol area README
- `protocol/plan/spec.md` — protocol draft
- `protocol/plan/todo.md` — protocol todo
- `tandem/README.md` — CLI/TUI area README
- `tandem/plan/spec.md` — CLI/TUI draft
- `tandem/plan/todo.md` — CLI/TUI todo
- `extensions/README.md` — integrations area README
- `extensions/plan/spec.md` — integrations draft
- `extensions/plan/todo.md` — integrations todo
- `extensions/pi-tandem/README.md` — Pi extension adapter README
- `extensions/pi-tandem/plan/spec.md` — Pi adapter spec
- `extensions/pi-tandem/plan/todo.md` — Pi adapter todo
