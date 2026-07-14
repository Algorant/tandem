# Tandem Project Plan

Status: draft  
Date: 2026-06-28

Tandem is a simple monorepo for a local-first human/agent coordination protocol, its CLI/TUI tooling, and lightweight agent/editor integrations.

## Project shape

```text
tandem/
├── AGENTS.md
├── README.md
├── plan/
│   ├── spec.md
│   └── todo.md
├── protocol/
│   ├── README.md
│   └── plan/
│       ├── spec.md
│       └── todo.md
├── tandem/
│   ├── README.md
│   └── plan/
│       ├── spec.md
│       └── todo.md
└── extensions/
    ├── README.md
    ├── plan/
    │   ├── spec.md
    │   └── todo.md
    └── pi-tandem/
        ├── README.md
        └── plan/
            ├── spec.md
            └── todo.md
```

## Naming model

- Product/protocol: **Tandem**
- Repository: `tandem`
- Project data directory: `.tandem/`
- Project config file: `.tandem/tandem.md`
- CLI binary: `tandem`
- TUI source area: `tandem/`
- Integrations area: `extensions/`
- Pi extension adapter: `pi-tandem`
- Work agreement object: `accord`
- User-facing CLI: `tandem`; reserve `td` for future/internal tool prefixes unless explicitly revisited

## Core idea

Tandem takes inspiration from Brainfile's file-based board model, but shifts the emphasis toward collaboration and orchestration between humans and agents.

The current baseline is not a blank-slate redesign. Tandem should use the general shape and useful features of the live Brainfile project as inspiration, then improve the flawed parts and adapt the language/UX to Tandem. Tandem does not require Brainfile import/migration or ongoing Brainfile nomenclature compatibility.

The important distinction is:

- **Tandem** is the project/protocol/tooling system.
- An **accord** is the explicit agreement for a unit of work: scope, deliverables, constraints, validation, evidence, and acceptance.

## Monorepo strategy

Keep protocol and CLI/TUI work together while the idea is still forming. Split later only if the boundaries become stable enough to justify separate repositories.

Current areas:

- `protocol/` — the protocol/spec source of truth: Tandem on-disk format, lifecycle, accord/review/log semantics, and local v3 direction inspired by Brainfile.
- `tandem/` — CLI + TUI design and implementation. The user-facing CLI is `tandem`; the current v0 CLI surface is implemented as a single Rust binary crate, and forward implementation focus is the interactive Rust + Ratatui/crossterm TUI.
- `extensions/` — agent/editor integrations. The first integration is `pi-tandem`, a lightweight Pi adapter over an installed `tandem` CLI.
- `plan/` — parent project coordination and cross-cutting decisions.

Do not overdesign the repository. For v0, keep CLI/TUI implementation under `tandem/`, keep integrations under `extensions/`, and do not add a root Rust workspace, `crates/`, standalone core crates, schemas, fixtures, CI, or dependency choices. Revisit only after implementation pressure proves the need.

## Locked v0 decisions

Protocol:

- Protocol version for the first v0 draft is `0.1.0`.
- Canonical workflow field is `state`; default states are `todo`, `in-progress`, `review`.
- New work items use `type: task`; root tasks default to flat sequential IDs such as `task-1`, while new first-class children default to parent-derived sequential IDs such as `task-103-1` and nested `task-103-1-1`.
- First-class document types are `task` and `decision`; decision documents are ADR-compatible durable records, do not need a lifecycle field in v0, and should not be split into a separate ADR type; custom types are config-only in v0.
- A first-class subtask is a normal `type: task` document linked to another task with `parentId`, retaining ordinary task workflow, ownership, accord, review, and completion behavior without a new type or relationship field.
- `parentId` remains the canonical hierarchy; ID shape alone never establishes parentage. Existing flat-ID children remain valid without migration.
- Child sequence allocation scans active board documents and completed logs and never reuses an ID. IDs are immutable; normal reparenting changes `parentId` without silently renaming IDs or rewriting references.
- Inline `subtasks:` checklist items are legacy and deprecated for new work; existing entries remain readable and preservable, while new trackable work should use child task documents.
- Epics are convention-only task documents using `type: task` plus `kind: epic` for broad outcome grouping; they use the same general `parentId` hierarchy, ordinary tasks may also parent children, loose context uses `references`, and v0 has no separate epic type or lifecycle.
- `accord` replaces Brainfile's contract concept with statuses: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`.
- Rules are structured objects. References may point to any Tandem document by ID.
- Completion warns but does not block on review/accord acceptance in v0.
- Archived markdown docs in `.tandem/logs/` are the completed-log source of truth; per-actor `.tandem/events/<actor_id>.jsonl` logs record minimal audit-only lifecycle events, with legacy `.tandem/events.jsonl` read during transition.
- Validation is built-in structural validation only for v0, with strict structure/core refs: unresolved `parentId`/`blockers` are errors; unresolved related `references` are warnings.
- No Brainfile import/migration command is required in v0.

CLI/TUI:

- v0 commands: `init`, `list`, `show`, `add`, `move`, `complete`, `log`, `search`, `accord`, `rules`, `decision`, `tui`.
- `tandem log` includes `list`, `show`, `search`; `tandem rules` includes `list`, `add`, `edit`, `delete`; `tandem accord` includes `ready`, `claim`, `deliver`, `accept`, `rework`, `block`, `fail`.
- Human-readable output is default using compact tables for list/search and labeled detail blocks for show/log/decision; all read commands support `--json` envelope objects.
- First CLI implementation language is Rust inside `tandem/`; the current implementation remains one `tandem` binary crate with `yaml-rust2` parsing, raw-source CLI mutations, and a `src/tui.rs` Ratatui/crossterm module.
- `tandem decision` supports `list`, `show`, and `add`.
- The TUI launches as `tandem tui` only in v0.
- First TUI MVP includes board mutations, Board/Review/Logs/Rules/Decisions views, theme support, mouse enabled by default without drag/drop, fixed default keybindings, styled-basic Markdown rendering, and a simple filtered-list Review queue. The current Board layout uses count-labeled state subviews with a full-width selected-state list rather than simultaneous columns.
- V0 CLI uses canonical command names and long flags only; no short aliases.
- Theme config loads in this order: built-in defaults, user TOML themes in `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, user config in `$XDG_CONFIG_HOME/tandem/config.toml` or `~/.config/tandem/config.toml`, then workspace selector/override at `.tandem/theme.toml`; Board display semantics such as project tag badge opt-ins load from user config and workspace `.tandem/config.toml`.
- Defer templates, schema CLI, MCP/hooks/auth, external archive integrations, schemas, and fixtures.
- Markdown planning docs stay canonical for now; migrate/dogfood Tandem documents after the TUI can manage them safely.
- `td` is reserved for future/internal tool prefixes; `tandem` remains the user-facing CLI.

Extensions:

- `extensions/pi-tandem` is the first v0 integration.
- `pi-tandem` is a thin Pi adapter over installed `tandem`, not a TypeScript Tandem protocol implementation.
- Extension tools must use `execFile`/argument arrays, prefer `tandem --json` read paths where supported, preserve human-readable mutation output, and provide diagnostics for missing `tandem`, missing `.tandem`, unsupported CLI surface, and command failures.
- Project-local extension testing comes before any global Pi config promotion.


## Near-term milestones

1. Reconcile Tandem protocol against live Brainfile protocol plus the local v3 proposal.
2. Build a feature parity/improvement matrix for live Brainfile CLI/TUI.
3. Stabilize protocol vocabulary and lifecycle.
4. Draft the detailed `tandem` command reference from the locked CLI surface.
5. Keep the existing `tandem/` Rust package layout stable unless implementation pressure proves a change is needed.
6. Treat the existing CLI v0 surface as complete for the current known scope; future CLI work should be explicit new features or bug fixes.
7. Continue the first Ratatui/crossterm TUI MVP from the current Board/Review/Logs/Rules/Decisions shell toward richer mutations and polish.
8. Add TUI accord/review/completion flows.
9. Keep Brainfile as a design reference only; no Brainfile import/migration work is required for v0.
10. Smoke `pi-tandem` locally as a Pi extension before considering canonical global Pi config promotion.
