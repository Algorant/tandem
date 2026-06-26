# Tandem Project Plan

Status: draft  
Date: 2026-06-26

Tandem is a simple monorepo for a local-first human/agent coordination protocol and its CLI/TUI tooling.

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
└── tandem-tui/
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
- CLI binary: `tdm`
- TUI source area: `tandem-tui/`
- Work agreement object: `accord`
- Future integration prefixes: `td` / `tdm`

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
- `tandem-tui/` — CLI + TUI design and later implementation. The user-facing CLI is `tdm`; CLI design comes first, then the interactive Rust + Ratatui TUI.
- `plan/` — parent project coordination and cross-cutting decisions.

Do not overdesign the repository. Do not add a root Rust workspace, `crates/`, standalone core crates, schemas, fixtures, CI, or dependency choices until they are explicitly requested or become the next necessary step.

## Locked v0 decisions

Protocol:

- Canonical workflow field is `state`; default states are `todo`, `in-progress`, `review`.
- New work items use `type: task` and sequential IDs such as `task-1`.
- First-class document types are `task` and `decision`; custom types are config-only in v0.
- `accord` replaces Brainfile's contract concept with statuses: `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, `blocked`.
- Rules are structured objects. References may point to any Tandem document by ID. Subtasks use parent-based sequential IDs.
- Completion warns but does not block on review/accord acceptance in v0.
- Archived markdown docs in `.tandem/logs/` are the completed-log source of truth; `.tandem/events.jsonl` records lifecycle events.
- Validation is built-in structural validation only for v0.
- No Brainfile import/migration command is required in v0.

CLI/TUI:

- v0 commands: `init`, `list`, `show`, `add`, `move`, `complete`, `log`, `search`, `accord`, `rules`, `decision`, `tui`.
- `tdm log` includes `list`, `show`, `search`; `tdm rules` includes `list`, `add`, `edit`, `delete`; `tdm accord` includes `ready`, `claim`, `deliver`, `accept`, `rework`, `block`, `fail`.
- Human-readable output is default; read commands should support `--json`.
- First CLI implementation language is Rust inside `tandem-tui/`.
- First TUI MVP includes board mutations, Board/Review/Logs/Rules/Decisions views, theme support, and mouse support.
- Defer templates, schema CLI, MCP/hooks/auth, and external archive integrations.

## Documentation contract

Every discrete aspect of Tandem must maintain documentation and remain synchronized with the parent project. No drift is allowed.

Minimum documentation set for each major area:

- `README.md` — purpose, scope, layout, current status, and links.
- `plan/spec.md` — design/specification for that area.
- `plan/todo.md` — accomplished/current/next task tracking for that area.

Parent docs are the coordination source of truth. When a naming, scope, architecture, lifecycle, or workflow decision changes, update the parent docs and every affected area doc in the same change.

## Near-term milestones

1. Reconcile Tandem protocol against live Brainfile protocol plus the local v3 proposal.
2. Build a feature parity/improvement matrix for live Brainfile CLI/TUI.
3. Stabilize protocol vocabulary and lifecycle.
4. Keep protocol, parent, and CLI/TUI documentation synchronized.
5. Decide the smallest useful `tdm` CLI surface when implementation begins.
6. Decide the minimal implementation layout only when coding starts.
7. Build protocol parsing and mutation primitives only after the spec is stable enough.
8. Build a read-only Ratatui board/logs prototype after core file semantics are clear.
9. Add accord/review/completion flows.
10. Keep Brainfile as a design reference only; no Brainfile import/migration work is required for v0.
