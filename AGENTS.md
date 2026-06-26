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
- **tdm**: intended CLI binary name.
- **td / tdm**: possible future shorthand prefixes for Pi extensions, tools, and integrations.

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
└── tandem-tui/
    ├── README.md          # TUI area README
    └── plan/
        ├── spec.md        # Rust/Ratatui TUI draft
        └── todo.md        # TUI todo
```

The repo is intentionally a monorepo for now. Do not split protocol/TUI into separate repositories unless explicitly asked.

## Documentation contract: no drift

Every discrete aspect of the project must have documentation. This includes the parent repository and each major sub-area such as `protocol/` and `tandem-tui/`.

Minimum documentation set for every discrete aspect:

- `README.md` — purpose, scope, layout, links to relevant docs, current status.
- `plan/spec.md` — design/specification for that aspect.
- `plan/todo.md` — accomplished/current/next task tracking for that aspect.

The parent docs are the coordination source of truth:

- `README.md`
- `plan/spec.md`
- `plan/todo.md`
- `AGENTS.md`

Sub-area docs must stay synchronized with the parent project docs. No drift is allowed.

When changing naming, layout, lifecycle, scope, architecture, or project direction, update all affected docs in the same change:

1. Parent README/spec/todo as applicable.
2. Relevant sub-area README/spec/todo.
3. `AGENTS.md` if the rule or workflow changes.

If docs disagree, treat parent docs as authoritative only long enough to reconcile the mismatch. Do not leave contradictory docs behind.

Before finishing documentation or planning changes, check for stale references and terminology, especially:

- old paths such as `protocol/spec.md` or `tandem-tui/spec.md`
- old project names or placeholder names
- `contract` where `accord` should be used
- `handoff` where `accord` should be used
- `done` column assumptions
- outdated CLI references such as `tandem` instead of `tdm`

## Current state

This project is currently in planning/specification mode. There is no Rust workspace or build system yet.

Primary planning documents:

- `plan/spec.md`
- `plan/todo.md`
- `protocol/README.md`
- `protocol/plan/spec.md`
- `protocol/plan/todo.md`
- `tandem-tui/README.md`
- `tandem-tui/plan/spec.md`
- `tandem-tui/plan/todo.md`

When changing direction, update the relevant README, `plan/todo.md`, and `plan/spec.md` files together.

## Naming rules

Use these names consistently unless the user explicitly changes them:

- Product/protocol: **Tandem**
- Repository: `tandem`
- Protocol data directory: `.tandem/`
- Protocol config file: `.tandem/tandem.md`
- CLI binary: `tdm`
- TUI area: `tandem-tui/`
- Work agreement object: `accord`
- Future integration prefixes: `td` / `tdm`

Avoid reintroducing `contract` except when discussing Brainfile compatibility or migration from Brainfile `contract` to Tandem `accord`.

## Design direction

Protocol:

- Use Markdown files with YAML frontmatter.
- Keep one active work document per file.
- Keep active work in `.tandem/board/`.
- Keep completed work in `.tandem/logs/`.
- Use `.tandem/events.jsonl` for append-only lifecycle history.
- Treat completion as an action/archive transition, not a persistent `done` column.
- Keep human workflow state, accord state, and review state separate.
- Preserve unknown fields and minimize file rewrites.

TUI:

- Target Rust + Ratatui.
- Do not port the Brainfile Ink TUI directly.
- Do not assume a `done` column for progress.
- Make review, accord status, logs, and validation prominent.
- Support themes from the beginning.
- Support mouse selection/scroll/click via a hit-map style event model.
- Keep keyboard-first ergonomics with vim-style and conventional bindings.

## Agent workflow

Before making changes:

1. Read this file.
2. Read parent docs: `README.md`, `plan/spec.md`, and `plan/todo.md`.
3. Read the relevant sub-area `README.md`, `plan/spec.md`, and `plan/todo.md`.
4. Keep changes scoped to the requested area.
5. If a decision changes project direction, update parent and area-specific planning docs in the same change.
6. Check for documentation drift before finalizing.

When adding implementation code later:

- Prefer small, reviewable commits/changes.
- Add tests or fixtures with protocol changes.
- Keep protocol parsing/mutation logic separate from TUI rendering logic.
- Do not create opaque state as the only source of truth.
- Update documentation and todos alongside code changes.

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
