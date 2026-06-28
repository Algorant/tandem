# AGENTS.md

Guidance for AI agents working in the Tandem repository.

## Project summary

Tandem is a local-first protocol and toolchain for human/agent project coordination.

It is inspired by Brainfile's file-based task model, but Tandem is intended to lean harder into collaboration, orchestration, review, and explicit work agreements between humans and agents.

Core concepts:

- **Tandem**: the product/protocol/tooling system.
- **accord**: the explicit agreement for a unit of work, replacing Brainfile's `contract` term.
- **review**: human/PM validation state, separate from accord state.
- **logs**: first-class completed-work history, not just a trash/archive folder.
- **tandem**: intended CLI binary name.
- **tandem**: user-facing CLI binary. **td** is reserved for future/internal tool prefixes unless explicitly revisited.

## Canonical project brief

Current direction is intentionally simple:

- Keep this as one parent monorepo named `tandem`.
- Keep exactly three major child areas for now: `protocol/`, `tandem/`, and `extensions/`.
- `protocol/` is the protocol/spec source of truth. Treat the protocol as the spec, not as a package/crate implementation area.
- The protocol baseline is inspired by the live Brainfile protocol plus the local v3 direction in `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md`: review state, complete/archive as an action, logs as first-class history, and accord/contract-to-state alignment. Tandem does not need Brainfile import/migration or long-term Brainfile nomenclature compatibility.
- `tandem/` is the current home for user-facing CLI + TUI planning and implementation. The CLI binary name is `tandem`; the directory remains `tandem/` until the user changes it.
- The v0 `tandem` CLI surface is implemented and considered complete for the current known scope; future CLI work should be explicit new features or bug fixes. Forward implementation focus is the Rust/Ratatui TUI, starting from the current Ratatui/crossterm Board shell. Aim for broad feature parity with live Brainfile CLI/TUI while fixing known flaws and not blindly copying every detail.
- The TUI target is Rust + Ratatui, but v0 implementation stays under `tandem/`. Do not turn the whole repository into a Rust workspace or introduce `crates/`, `tandem-core`, `clap`, schemas, fixtures, CI, or other structure in v0.
- `extensions/` is the scoped home for agent/editor integrations. The first integration is `extensions/pi-tandem/`, a lightweight Pi adapter over an installed `tandem` CLI; extension code must not duplicate Tandem protocol parsing or mutation behavior.
- Prefer the smallest next useful step. Proposals are welcome, but mark them as proposals/open questions rather than encoding them as settled decisions.
- Do not rename directories, move specs out of `plan/`, or collapse/expand the repo layout unless the orchestrator explicitly delegates that change.

## Locked v0 decisions

Protocol:

- Canonical workflow field: `state` / `states`.
- Default active states: `todo`, `in-progress`, `review`.
- Protocol version for the first v0 draft: `0.1.0`.
- Default task identity: `type: task` with sequential IDs such as `task-1`.
- First-class document types: `task` and `decision`; decision documents do not need a lifecycle field in v0.
- Custom document types: allowed in config only; no v0 type-management CLI.
- Work agreement object: `accord`.
- Canonical accord statuses: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`.
- Rules: structured objects with stable IDs, e.g. `{ id, rule, source? }`.
- References: `parentId`, blockers, and related references may point to any Tandem document by ID.
- Subtask IDs: parent-based sequential IDs such as `task-1-1`.
- Completion: `tandem complete` warns about missing review/accord acceptance but allows completion in v0.
- Events: `.tandem/events.jsonl` stores minimal audit-only lifecycle records requiring `ts`, `event`, `id`, and `summary`.
- Completed logs: archived markdown docs in `.tandem/logs/` are the primary source of truth; events enrich timeline/audit.
- Validation/lint: built-in structural validation only in v0; unresolved `parentId`/`blockers` are errors, while unresolved related `references`/rule sources and completion-policy issues are warnings.
- Brainfile migration/import: no v0 requirement and no required command.

CLI/TUI:

- v0 CLI commands: `init`, `list`, `show`, `add`, `move`, `complete`, `log`, `search`, `accord`, `rules`, `decision`, `tui`.
- v0 `tandem log`: `list`, `show`, `search` only.
- v0 `tandem rules`: `list`, `add`, `edit`, `delete`.
- v0 `tandem accord`: `ready`, `claim`, `deliver`, `accept`, `rework`, `block`, `fail`.
- CLI output: human-readable by default using compact tables for list/search and labeled detail blocks for show/log/decision; `--json` envelope objects for all read commands.
- V0 CLI alias policy: canonical command names and long flags only; no short aliases.
- `tandem decision`: `list`, `show`, `add`.
- First CLI implementation language: Rust, inside `tandem/`.
- TUI invocation: `tandem tui` only in v0.
- First TUI MVP: includes board mutations immediately.
- TUI top-level views: Board, Review, Logs, Rules, Decisions.
- Theme and mouse support are part of the first TUI MVP.
- Mouse is enabled by default for click/scroll/tab/action-button interactions; drag/drop is excluded from v0.
- Theme config loading order: built-in defaults, user TOML themes in `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, workspace selector/override at `.tandem/theme.toml`.
- V0 keybindings are fixed defaults; custom keymap config is deferred.
- V0 Markdown rendering is styled basics.
- V0 Review queue is a simple filtered list, not hard-coded workflow sections.
- Deferred from v0: templates, schema CLI, MCP/hooks/auth, external archive integrations, schemas, fixtures, and root Rust workspace layout.


## Repository layout

```text
.
├── AGENTS.md
├── README.md              # parent project README
├── plan/
│   ├── spec.md            # parent project plan/spec
│   └── todo.md            # parent project todo
├── protocol/
│   ├── README.md          # protocol area README
│   └── plan/
│       ├── spec.md        # Tandem protocol draft
│       └── todo.md        # protocol todo
├── tandem/
│   ├── README.md          # CLI/TUI area README
│   └── plan/
│       ├── spec.md        # CLI + Rust/Ratatui TUI draft
│       └── todo.md        # CLI/TUI todo
└── extensions/
    ├── README.md          # integrations area README
    ├── plan/
    │   ├── spec.md        # integrations area draft
    │   └── todo.md        # integrations todo
    └── pi-tandem/         # Pi adapter over tandem CLI
```

The repo is intentionally a monorepo for now. Do not split protocol/CLI/TUI/extensions into separate repositories unless explicitly asked.


## Current state

This project is currently in planning/specification plus implementation mode. There is no root Rust workspace; CLI/TUI implementation lives in the single `tandem/` Rust binary crate that builds `tandem`, and integration work lives under `extensions/`.

Primary planning documents:

- `plan/spec.md`
- `plan/todo.md`
- `protocol/README.md`
- `protocol/plan/spec.md`
- `protocol/plan/todo.md`
- `tandem/README.md`
- `tandem/plan/spec.md`
- `tandem/plan/todo.md`
- `extensions/README.md`
- `extensions/plan/spec.md`
- `extensions/plan/todo.md`
- `extensions/pi-tandem/README.md`
- `extensions/pi-tandem/plan/spec.md`
- `extensions/pi-tandem/plan/todo.md`

## Naming rules

Use these names consistently unless the user explicitly changes them:

- Product/protocol: **Tandem**
- Repository: `tandem`
- Protocol data directory: `.tandem/`
- Protocol config file: `.tandem/tandem.md`
- CLI binary: `tandem`
- CLI/TUI area: `tandem/`
- Integrations area: `extensions/`
- Pi extension adapter: `pi-tandem`
- Work agreement object: `accord`
- User-facing CLI: `tandem`; reserve `td` for future/internal tool prefixes

Avoid reintroducing `contract` except when discussing Brainfile design mapping from `contract` to Tandem `accord`.

## Design direction

Protocol:

- Start from Brainfile's live protocol and command shape, then adapt it into Tandem vocabulary and v3 improvements.
- Use Markdown files with YAML frontmatter.
- Keep one active work document per file.
- Keep active work in `.tandem/board/`.
- Keep completed work in `.tandem/logs/`.
- Use `.tandem/events.jsonl` for append-only lifecycle history.
- Treat completion as an action/archive transition, not a persistent `done` column.
- Keep human workflow state, accord state, and review state separate.
- Preserve unknown fields and minimize file rewrites.
- Use the local v3 Brainfile proposal as directional input for review/logs/completion behavior.

CLI/TUI:

- Keep CLI and TUI planning together in `tandem/` for now.
- The CLI binary name is `tandem`.
- Target Rust + Ratatui for the interactive TUI.
- Do not port the Brainfile Ink TUI directly.
- Do not assume a `done` column for progress.
- Make review, accord status, logs, and validation prominent.
- Support themes from the beginning.
- Support mouse selection/scroll/click via a hit-map style event model.
- Keep keyboard-first ergonomics with vim-style and conventional bindings.
- Keep v0 implementation under `tandem/`; treat package/module layout and dependency choices inside that area as open until coding begins.
- Evaluate live Brainfile CLI/TUI features for parity, then decide what to keep, rename, improve, or intentionally omit.

Extensions:

- Keep integrations under `extensions/` for now.
- Model `pi-tandem` after `pi-web-tools`: a thin Pi adapter over an installed CLI.
- Use `execFile`/argument arrays and avoid shell interpolation.
- Do not duplicate Tandem protocol parsing or mutation behavior in TypeScript; call `tandem` and keep behavior in the CLI/protocol.
- Test project-local extension behavior first; promote to canonical global Pi config only in an explicit later task.

## Agent workflow

Before making changes:

1. Read this file.
2. Inspect the files directly relevant to the requested area.
3. Keep changes scoped to the requested area.

When adding implementation code later:

- Prefer small, reviewable commits/changes.
- Add tests with protocol changes when implementation exists; do not add schemas or fixtures in v0.
- Keep protocol parsing/mutation logic separate from TUI rendering logic and extension adapter logic.
- Do not create opaque state as the only source of truth.

## File editing rules

- Keep Markdown clear and concise.
- Update todo checkboxes when tasks are completed or superseded.
- Preserve existing terminology unless intentionally changing it.
- Avoid large rewrites of specs unless the user asks for a rewrite.
- Do not commit secrets, auth tokens, session logs, local caches, or generated build artifacts.
- `.pi/` is local runtime state and should remain ignored.

## Git/GitHub

Remote repository:

- `git@github.com:Algorant/tandem.git`
- private GitHub repo: `Algorant/tandem`

Do not push unless the user asks or the current task clearly includes repository synchronization. If you do push, summarize the commit hash and remote branch.
