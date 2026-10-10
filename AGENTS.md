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
- Repository `protocol/` Markdown is the normative protocol source of truth. `tandem/src/protocol/` is its executable Rust implementation, not a second specification.
- The protocol baseline is inspired by the live Brainfile protocol plus the local v3 direction in `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md`: review state, complete/archive as an action, logs as first-class history, and accord/contract-to-state alignment. Tandem does not need Brainfile import/migration or long-term Brainfile nomenclature compatibility.
- `tandem/` is the canonical home for the shared Rust CLI + TUI app. The user-facing command is `tandem`; do not reintroduce `tdm` or split the app unless explicitly asked.
- The Rust architecture is implemented as `protocol`, `project`, `app`, `cli`, and `tui` modules in one binary crate. `project::TandemProject` owns concrete `.tandem/` discovery and filesystem safety; shared `app` operations coordinate protocol rules and project I/O; CLI and TUI are peer interfaces. `main.rs` and `tui/mod.rs` are wiring roots, not protocol or persistence owners.
- The TUI target is Rust + Ratatui, but v0 implementation stays under `tandem/`. Do not turn the whole repository into a Rust workspace or introduce `crates/`, `tandem-core`, schemas, fixtures, CI, or other structure in v0.
- `extensions/` is the scoped home for future agent/editor integrations. It currently holds no integration: the `pi-tandem` adapter was retired (`decision-10`), and the `tandem` CLI with `--json` is the integration surface. Integration code must not duplicate Tandem protocol parsing or mutation behavior.
- Prefer the smallest next useful step. Proposals are welcome, but mark them as proposals/open questions rather than encoding them as settled decisions.
- Do not rename directories, move specs out of `plan/`, or collapse/expand the repo layout unless the orchestrator explicitly delegates that change.

## Locked decisions (current)

These summarize the current implementation: protocol `0.5.0` (`decision-9`) and the clap-derived `tandem` CLI. `protocol/README.md` is normative and `tandem --help` prints the authoritative command tree; if this list disagrees with either, they win and this file is stale.

Protocol:

- Canonical workflow field: `state` / `states`. Active states are exactly `todo`, `in-progress`, and `validation`.
- Current protocol version: `0.5.0`. A `0.3.0` or `0.4.0` board is converted once by `tandem migrate`; ordinary commands on an older board fail with the required version and never upgrade implicitly. There is no `tandem upgrade`.
- Board layout: `.tandem/tandem.md`, `tasks/`, `decisions/`, `rules/`, `logs/`, and per-actor `events/`. Every record has a permanent `uid`. In Git the board is the main worktree's ignored `.tandem/` and syncs through the repository's `tandem` branch, independent of source commits; Tandem syncs it itself.
- First-class document types are `task` and `decision`; Decisions are ADR-compatible durable records with a `status` (`proposed`, `accepted`, `rejected`, `deprecated`, `superseded`) and no workflow `state`. Only `task` and `decision` can be created.
- Default task identity: `type: task`; every Epic and Task uses the global flat `task-N` namespace, including a direct Task beneath an Epic. Only a Subtask directly beneath a Task uses the parent-derived `task-N-M` form. IDs are immutable.
- Hierarchy roles are derived from resolved documents, never ID shape: an Epic is `type: task` plus `kind: epic` and is root-only; a Task is a normal task that is root-level, has a generic non-task parent, or is directly parented by an Epic; a Subtask is a normal task directly parented by a Task and cannot have children. Direct Epic children use relationship `epic-task`, Task children use `subtask`, and decision parents use generic `parent`. Role-specific IDs are strict: a role/ID mismatch is a structural error, and reparenting that would change a role (such as clearing a Subtask's parent) is refused before writing.
- Task `kind` is `epic`, `research`, or `papercut`; tags are never kinds. A papercut needs only a title, defaults to low priority, and is never a Subtask.
- Work agreement object: `accord`. Accord statuses: `ready`, `claimed`, `delivered`, `rework`, `blocked`, and terminal `accepted` or `failed`. Typed `links` (`relates-to`, `duplicates`, `fixed-by`/`fixes`, `supersedes`) are separate from hierarchy and loose `references`.
- Rules: Markdown records with composite IDs such as `always-12`, a category, optional `source`, and the rule text as the body.
- References: `parentId` and blockers must resolve (errors); related `references` may point to any Tandem document by ID or an absolute `http(s)` URL (unresolved IDs warn).
- Lifecycle: no command sets `state` directly. `accord claim|rework|release`, `review`, `complete`, and `cancel` change it. `accord deliver` requires `--summary` and non-empty `--evidence`, sets `accord.status: delivered`, and leaves `state` unchanged. `tandem review <id> --criterion <exact acceptance criterion> --note <text>` is the only route to `validation`; review status is not stored. `tandem complete` accepts a delivered Accord and archives atomically, and completing an undelivered Task warns but succeeds. `tandem cancel` archives with `resolution.outcome: canceled` and rejects active descendants.
- Events: per-actor `.tandem/events/<actor-id>.jsonl` ledgers hold minimal audit-only lifecycle records requiring `ts`, `event`, `id`, `actor`, and `seq`. Actor identity is checkout-local (`<git-dir>/tandem-actor-id`, or `.tandem/actor-id` outside Git); Tandem owns it and integrations must not generate, copy, parse, or globally inject it.
- Completed logs: archived records in `.tandem/logs/` are the primary source of truth; events enrich timeline/audit.
- Sync: a local record that parses but would make the shared board invalid is a held edit; reads use the shared version with a warning, and `tandem sync resolve <id> --keep remote` repairs it.
- Validation/lint: built-in structural validation only; unresolved `parentId`/`blockers` are errors, unresolved related references and rule sources are warnings.
- Brainfile migration/import is not a requirement and has no command.

CLI/TUI:

- CLI commands (30 leaves): `init`; `add task|decision`; `show`; `assignment`; `list`; `search`; `update`; `accord claim|deliver|rework|block|resume|release|fail`; `review`; `complete`; `cancel`; `link add|remove`; `sync`, `sync status|resolve`; `migrate`; `rules list|add|edit|delete`; `tui`; `web`. There is no `move`, `log`, `decision`, `papercut`, `upgrade`, or `version` command, no `accord ready|accept`, and no Task `update --status` (Decision `update --status` exists).
- Completed history is read with `list|search --scope archived|all` and `show`; Decisions with `list --type decision`.
- Global flags: `-j/--json`, `-h/--help`, `-V/--version`. Other flags are long-form only. Mutations are human-readable by default; every command supports `--json` with `{ "ok", "data", "warnings" }` envelopes.
- The CLI is Rust (clap-derived) inside `tandem/`.
- TUI invocation: `tandem tui`. Board mutations are part of the TUI.
- TUI top-level views: Board, Logs, Rules, Decisions; validation work is presented through the Board Validation state and actions.
- Theme and mouse support are included. Mouse is enabled by default for click/scroll/tab/action-button interactions; drag/drop is excluded.
- Theme config loading order: built-in defaults, user TOML themes in `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, user config in `$XDG_CONFIG_HOME/tandem/config.toml` or `~/.config/tandem/config.toml`, then workspace selector/override at `.tandem/theme.toml`; Board display settings such as project tag badge opt-ins load from user config and workspace `.tandem/config.toml`.
- Keybindings are fixed defaults; custom keymap config is deferred.
- Markdown rendering is styled basics.
- Deferred: templates, schema CLI, MCP/hooks/auth, external archive integrations, schemas, fixtures, and root Rust workspace layout.


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
    └── plan/
        ├── spec.md        # integrations area draft
        └── todo.md        # integrations todo
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

## Naming rules

Use these names consistently unless the user explicitly changes them:

- Product/protocol: **Tandem**
- Repository: `tandem`
- Protocol data directory: `.tandem/`
- Protocol config file: `.tandem/tandem.md`
- CLI binary: `tandem`
- CLI/TUI area: `tandem/`
- Integrations area: `extensions/`
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
- Keep active work in `.tandem/tasks/` and `.tandem/decisions/`.
- Keep completed work in `.tandem/logs/`.
- Use per-actor `.tandem/events/<actor_id>.jsonl` logs for append-only lifecycle history; readers aggregate those logs.
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
- Keep implementation under the single `tandem/` binary crate. Preserve the implemented dependency direction: `protocol` owns meaning, `project` owns concrete files, `app` owns shared operations, and peer `cli`/`tui` interfaces consume those layers.
- Evaluate live Brainfile CLI/TUI features for parity, then decide what to keep, rename, improve, or intentionally omit.

Extensions:

- Keep integrations under `extensions/` for now.
- Any future integration is a thin adapter over an installed `tandem` CLI: use `execFile`/argument arrays, avoid shell interpolation, and consume CLI JSON.
- Do not duplicate Tandem protocol parsing or mutation behavior in an integration; call `tandem` and keep behavior in the CLI/protocol.

## Protocol architecture refactor campaign (completed)

The `refactor/protocol-architecture` campaign is **complete**. Campaign Epic
`task-146` is archived in `.tandem/logs/` and the campaign branch no longer
exists. There is no implementation freeze on `main`.

The campaign established five ownership boundaries that remain in force:
`protocol`, `project`, `app`, `cli`, and `tui`. See
[`plan/refactor_campaign_baseline.md`](plan/refactor_campaign_baseline.md) and
[`plan/refactor_spec.md`](plan/refactor_spec.md) for the historical record and
`decision-8` for the architecture decision itself.

One campaign practice survives as an ongoing repository rule:

- Delegate only independently reviewable Tasks, in isolated Task
  worktrees/branches, reviewed before integration.

The campaign also required human terminal validation for every visible TUI
change. That blanket requirement is retired; rule `always-12` governs when to
request review, based on whether you can verify the acceptance criteria rather
than on whether the work is visual.

## Validating TUI changes

A rendered pane is evidence. Spawn a Herdr pane and read the result with
`herdr pane read --format ansi` to check layout, counts, wrapping, colors, and
attributes. Build release, not debug: debug builds render slowly enough to
distort flicker, resize, and latency observations. `just dev` builds release
and opens a disposable Git-backed sandbox; `just dev-project` explicitly targets
the real project records. Use `just dev-check` for native tests and a Git-backed
workflow smoke test.

A snapshot cannot settle temporal behavior. Flicker, resize tearing, redraw
latency, and cursor ghosting need a human at a terminal. So do density,
readability over a long session, and any question about what the design should
be rather than whether it matches the intent.

For delegated TUI work, configure the repository's Git-local preview slot so the
user runs only `just dev` from the normal checkout. Route it to the delegated
code and a safe fixture, report that no extra setup is needed, and clear the
route during cleanup.

## Agent workflow

Before making changes:

1. Read this file.
2. Run `tandem rules list` and follow the project's active rules. They are
   authoritative operating guidance, not background reading, and they change
   more often than this file. Read them at the start of a session, not after
   deciding how to work.
3. Run `tandem list` to see what is on the Board, and `tandem show <id>` for
   anything related to the requested area. Existing tasks, blockers, and
   in-flight work change what the right next step is.
4. Inspect the files directly relevant to the requested area.
5. Keep changes scoped to the requested area.

If two rules conflict, or a rule contradicts this file, say so and get it
resolved rather than silently picking one. Do not infer which rule wins.

When recording durable decisions:

- Use `tandem add decision` to create first-class `type: decision` documents.
- Do not model decisions as task lifecycle states, accord statuses, completed logs, or a separate `adr` document type.
- For ADR-compatible records, include body sections such as Status, Context, Decision, Consequences, and Supersession; optional status/supersession metadata is decision record metadata, not workflow `state`.

When changing implementation code:

- Prefer small, reviewable commits/changes.
- Add tests with protocol changes; do not add schemas or fixtures in v0.
- Change normative semantics in `protocol/` first, then update `tandem/src/protocol/`; do not infer roles, IDs, lifecycle, accord, review, or event rules in `project`, CLI, TUI, or extensions.
- Route durable CLI and TUI mutations through shared `app` operations over `project::TandemProject`.
- Keep any integration CLI-only: it may build argument arrays and consume CLI JSON, but must never parse or mutate Tandem Markdown/frontmatter or reclassify protocol relationships.
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
