# AGENTS.md

Guidance for AI agents working in the Tandem repository.

## Project summary

Tandem is a local-first protocol and toolchain for human/agent project coordination.

It is inspired by Brainfile's file-based task model, but Tandem is intended to lean harder into collaboration, orchestration, review, and explicit work agreements between humans and agents.

Core concepts:

- **Tandem**: the product/protocol/tooling system.
- **accord**: the explicit agreement for a unit of work, replacing Brainfile's `contract` term.
- **decision**: a first-class durable project/product/architecture choice; Tandem decisions are ADR-compatible records without a separate ADR type.
- **review**: human/PM validation state, separate from accord state.
- **logs**: first-class completed-work history, not just a trash/archive folder.
- **tandem**: user-facing CLI/TUI binary and Rust app crate. **td** is reserved for future/internal tool prefixes unless explicitly revisited.

## Canonical project brief

Current direction is intentionally simple:

- Keep this as one parent monorepo named `tandem`.
- Keep exactly three major child areas for now: `protocol/`, `tandem/`, and `extensions/`.
- `protocol/` is the protocol/spec source of truth. Treat the protocol as the spec, not as a package/crate implementation area.
- The protocol baseline is inspired by the live Brainfile protocol plus the local v3 direction in `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md`: review state, complete/archive as an action, logs as first-class history, and accord/contract-to-state alignment. Tandem does not need Brainfile import/migration or long-term Brainfile nomenclature compatibility.
- `tandem/` is the canonical home for the shared Rust CLI + TUI app. The user-facing command is `tandem`; do not reintroduce `tdm` or split the app unless explicitly asked.
- The prior v0 `tandem` CLI surface is implemented, but the accepted strict Epic → Task → Subtask correction is not complete until its delegated protocol, CLI, TUI, and integration tasks deliver. Do not describe the current CLI/TUI implementation as complete while that work remains open. Aim for broad feature parity with live Brainfile CLI/TUI while fixing known flaws and not blindly copying every detail.
- The TUI target is Rust + Ratatui, but v0 implementation stays under `tandem/`. Do not turn the whole repository into a Rust workspace or introduce `crates/`, `tandem-core`, `clap`, schemas, fixtures, CI, or other structure in v0.
- `extensions/` is the scoped home for agent/editor integrations. The first integration is `extensions/pi-tandem/`, a lightweight Pi adapter over an installed `tandem` CLI; extension code must not duplicate Tandem protocol parsing or mutation behavior.
- Prefer the smallest next useful step. Proposals are welcome, but mark them as proposals/open questions rather than encoding them as settled decisions.
- Do not rename directories, move specs out of `plan/`, or collapse/expand the repo layout unless the orchestrator explicitly delegates that change.

## Locked v0 decisions

Protocol:

- Canonical workflow field: `state` / `states`.
- Default active states: `todo`, `in-progress`, `review`.
- Protocol version for the first v0 draft: `0.1.0`.
- Default task identity: `type: task`; every Epic and Task uses the global flat `task-N` namespace, including a direct Task beneath an Epic. Only a Subtask directly beneath a Task uses the parent-derived `task-N-M` form.
- First-class document types: `task` and `decision`; decision documents are ADR-compatible durable records and do not need a lifecycle field in v0.
- Custom document types: allowed in config only; no v0 type-management CLI.
- Hierarchy roles are derived from resolved documents, never ID shape: an Epic is `type: task` plus `kind: epic`; a Task is a normal task that is root-level, has a generic non-task parent, or is directly parented by an Epic; a Subtask is a normal task directly parented by a Task. Direct Epic children use relationship `epic-task`, Task children use `subtask`, and decision/custom-document parents use generic `parent`.
- Work agreement object: `accord`.
- Canonical accord statuses: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`.
- Rules: structured objects with stable IDs, e.g. `{ id, rule, source? }`.
- References: `parentId`, blockers, and related references may point to any Tandem document by ID.
- Role-specific IDs are strict: Epics and Tasks allocate the next global `task-N` across active board documents and completed logs; Subtasks allocate the next `task-N-M` suffix beneath their Task across board and logs without reuse. Resolved documents define roles, while the role constrains valid ID shape. Direct Epic Tasks with hierarchical IDs and Subtasks with global IDs are invalid; there is no decision-4 compatibility exception.
- Completion: `tandem complete` warns about missing review/accord acceptance but allows completion in v0.
- Events: per-actor `.tandem/events/<actor_id>.jsonl` logs store minimal audit-only lifecycle records requiring `ts`, `event`, `id`, `summary`, `actor`, and `seq`; legacy `.tandem/events.jsonl` remains readable during transition.
- Completed logs: archived markdown docs in `.tandem/logs/` are the primary source of truth; events enrich timeline/audit.
- Validation/lint: built-in structural validation only in v0; unresolved `parentId`/`blockers` are errors, while unresolved related `references`/rule sources and completion-policy issues are warnings. Epics with `parentId`, children beneath Subtasks, and reparenting that would create either condition are structural errors; Subtasks cannot have children.
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
- Theme config loading order: built-in defaults, user TOML themes in `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, user config in `$XDG_CONFIG_HOME/tandem/config.toml` or `~/.config/tandem/config.toml`, then workspace selector/override at `.tandem/theme.toml`; Board display settings such as project tag badge opt-ins load from user config and workspace `.tandem/config.toml`.
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

## Epic, Task, and Subtask convention for agents

- Model epics as ordinary tasks with `type: task` and `kind: epic`; do not invent `type: epic`, `epic-N` IDs, ADR-style epic records, custom folders, or special workflow states. Epics are root-only and cannot have `parentId`.
- Decompose an Epic into independently managed Tasks. Each direct Epic child links through `parentId`, remains a Task with a global `task-N` ID, and has relationship `epic-task`—never `subtask`.
- Epics are planning/grouping roots and are not delegated. Delegate a Task to Worker A; its Subtask documents are Worker A's bounded execution checklist, projected into `pi-todos` and executed directly without independently delegating them or spawning Worker B. Parent review remains at the delegated Task boundary.
- Create a Subtask only beneath a Task and only for smaller lifecycle-bearing checklist work. It uses `task-N-M`, cannot have children, and is not an independent delegation unit. A Task with a decision/custom-document parent remains a global-ID Task and may have Subtasks.
- Derive Epic, Task, Subtask, `epic-task`, `subtask`, and generic `parent` classifications from resolved documents, then validate the required role-specific ID form. Never infer role from ID shape alone.
- Create or inspect the parent before adding children because unresolved parents are errors. Reject a parented Epic, a child beneath a Subtask, every role/ID mismatch, and reparenting that changes a document's role or invalidates its ID.
- Use `references` for loose related context such as decisions, sibling tasks, or completed logs. References are not hierarchy and unresolved references are warnings.
- Inline `subtasks` are legacy checklist data, not the canonical Worker A checklist; do not author them for lifecycle-bearing work.
- Complete/archive epics with the normal task completion flow only after their Tasks are completed, intentionally canceled/superseded, or the project owner decides the epic is done. Do not create a persistent `done` state.
- Keep decisions/ADR-style documents for durable decisions only; do not use them as a substitute for epic tracking.

## Design direction

Protocol:

- Start from Brainfile's live protocol and command shape, then adapt it into Tandem vocabulary and v3 improvements.
- Use Markdown files with YAML frontmatter.
- Keep one active work document per file.
- Keep active work in `.tandem/board/`.
- Keep completed work in `.tandem/logs/`.
- Use per-actor `.tandem/events/<actor_id>.jsonl` logs for append-only lifecycle history; readers should aggregate those logs plus legacy `.tandem/events.jsonl` if present.
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

When recording durable decisions:

- Use `tandem_decision` / `tandem decision add` to create first-class `type: decision` documents.
- Do not model decisions as task lifecycle states, accord statuses, completed logs, or a separate `adr` document type.
- For ADR-compatible records, include body sections such as Status, Context, Decision, Consequences, and Supersession; optional status/supersession metadata is decision record metadata, not workflow `state`.

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
