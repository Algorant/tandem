# Tandem Project Plan

Status: draft  
Date: 2026-06-26

Tandem is a monorepo for a local-first human/agent coordination protocol and its terminal UI.

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

The important distinction is:

- **Tandem** is the project/protocol/tooling system.
- An **accord** is the explicit agreement for a unit of work: scope, deliverables, constraints, validation, evidence, and acceptance.

## Monorepo strategy

Keep protocol and TUI work together while the idea is still forming. Split later only if the boundaries become stable enough to justify separate repositories.

Initial areas:

- `protocol/` — on-disk format, lifecycle, schemas, CLI concepts, Brainfile import plan.
- `tandem-tui/` — Rust/Ratatui TUI design and later implementation.
- `plan/` — parent project coordination and cross-cutting decisions.

## Documentation contract

Every discrete aspect of Tandem must maintain documentation and remain synchronized with the parent project. No drift is allowed.

Minimum documentation set for each major area:

- `README.md` — purpose, scope, layout, current status, and links.
- `plan/spec.md` — design/specification for that area.
- `plan/todo.md` — accomplished/current/next task tracking for that area.

Parent docs are the coordination source of truth. When a naming, scope, architecture, lifecycle, or workflow decision changes, update the parent docs and every affected area doc in the same change.

## Near-term milestones

1. Stabilize protocol vocabulary and lifecycle.
2. Create example Tandem workspaces/fixtures.
3. Define Rust workspace/crate layout.
4. Implement protocol parsing and minimal mutation primitives.
5. Build a read-only Ratatui board/logs prototype.
6. Add accord/review/completion flows.
7. Add import compatibility from Brainfile.
