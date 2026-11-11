# AGENTS.md

Guidance for AI agents working in the Tandem repository.

## Ideas worth borrowing

- Accords: an explicit claim/deliver/rework/accept agreement kept separate from workflow state and review — `protocol/README.md` (Accord and archive).
- Plain Markdown board with per-actor append-only event ledgers, so concurrent agents never contend on one log — `protocol/README.md` (Events), `tandem/src/project/events.rs`.
- Board sync on its own Git branch, independent of source commits, with held edits instead of a broken shared board — `protocol/README.md` (Sync), `tandem/src/project/sync.rs`.
- One binary with strict layers: `protocol` owns meaning, `project` owns files, `app` owns operations, and `cli`/`tui` are peer interfaces — `tandem/src/`.
- Rules as Markdown records that agents re-read at session start — `tandem/src/project/rules.rs`, `tandem/src/project/agent_files.rs`.

## Project summary

Tandem is a local-first protocol and toolchain for human/agent project coordination.

It is inspired by [Brainfile](https://brainfile.md/)'s file-based task model, but Tandem is intended to lean harder into collaboration, orchestration, review, and explicit work agreements between humans and agents.

Core concepts:

- **Tandem**: the product/protocol/tooling system.
- **accord**: the explicit agreement for a unit of work, replacing Brainfile's `contract` term.
- **decision**: a first-class durable project/product/architecture choice; Tandem decisions are ADR-compatible records without a separate ADR type.
- **review**: human/PM validation state, separate from accord state.
- **logs**: first-class completed-work history, not just a trash/archive folder.
- **tandem**: user-facing CLI/TUI binary and Rust app crate. **td** is reserved for future/internal tool prefixes unless explicitly revisited.

## Canonical project brief

Current direction is intentionally simple:

- Keep this one monorepo: `protocol/` for the normative protocol, `tandem/` for the Rust implementation, and `docs/` with `site/` for the public documentation.
- Repository `protocol/` Markdown is the normative protocol source of truth. `tandem/src/protocol/` is its executable Rust implementation, not a second specification.
- The protocol baseline is inspired by the [Brainfile](https://brainfile.md/) protocol: review state, complete/archive as an action, logs as first-class history, and accord/contract-to-state alignment. Tandem does not need Brainfile import/migration or long-term Brainfile nomenclature compatibility.
- `tandem/` is the canonical home for the shared Rust CLI + TUI app. The user-facing command is `tandem`; do not reintroduce `tdm` or split the app unless explicitly asked.
- The Rust architecture is implemented as `protocol`, `project`, `app`, `cli`, and `tui` modules in one binary crate. `project::TandemProject` owns concrete `.tandem/` discovery and filesystem safety; shared `app` operations coordinate protocol rules and project I/O; CLI and TUI are peer interfaces. `main.rs` and `tui/mod.rs` are wiring roots, not protocol or persistence owners.
- The TUI target is Rust + Ratatui, but v0 implementation stays under `tandem/`. Do not turn the whole repository into a Rust workspace or introduce `crates/`, `tandem-core`, schemas, fixtures, CI, or other structure in v0.
- The `tandem` CLI with `--json` is the integration surface. The `pi-tandem` adapter was retired (`decision-10`); integration code must not duplicate Tandem protocol parsing or mutation behavior.
- Prefer the smallest next useful step. Proposals are welcome, but mark them as proposals/open questions rather than encoding them as settled decisions.
- Do not rename directories or collapse/expand the repo layout unless the orchestrator explicitly delegates that change.

## Where the rules live

`protocol/README.md` is normative for board meaning, `tandem --help` is the authoritative command tree, and `docs/` is the public behaviour. Read those instead of a copy kept here. The invariants an agent most often needs:

- Protocol `0.5.0`; `tandem migrate` converts an older board once, and ordinary commands never upgrade implicitly.
- The board is the main worktree's ignored `.tandem/`, synced by Tandem through the repository's `tandem` branch, independent of source commits.
- Workflow states are `todo`, `in-progress` and `validation`; no command sets `state` directly.
- Accord statuses are `ready`, `claimed`, `delivered`, `rework`, `blocked`, and terminal `accepted`/`failed`; `deliver` needs a summary and evidence; `complete` accepts and archives; `cancel` archives as canceled.
- `review` is the only route into `validation`, and its status is not stored.
- Records are Markdown with YAML frontmatter, a permanent `uid`, and immutable IDs.

## Repository layout

```text
.
├── AGENTS.md
├── README.md
├── protocol/          # normative protocol: README.md, assignment.md, links.md
├── tandem/            # Rust binary crate: src/{protocol,project,app,cli,tui}, RELEASE.md, AUR.md
├── docs/              # public Markdown documentation source
├── site/              # Astro Starlight site built from docs/
├── scripts/           # release and development helpers
└── justfile
```

The Rust package lives under `tandem/`; there is no root Cargo workspace. Do not split the protocol, CLI/TUI and docs into separate repositories unless explicitly asked.

## Current state

The CLI and TUI ship as 0.17.x against protocol 0.5.0. Development happens in the single Rust crate under `tandem/`; the public documentation is `docs/`, published by `site/`. `protocol/README.md` and `tandem --help` win over any summary in this file.

## Naming rules

Use these names consistently unless the user explicitly changes them:

- Product/protocol: **Tandem**
- Repository: `tandem`
- Protocol data directory: `.tandem/`
- Protocol config file: `.tandem/tandem.md`
- CLI binary: `tandem`
- CLI/TUI area: `tandem/`
- Public documentation: `docs/` source, `site/` build
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
- Treat [Brainfile](https://brainfile.md/) as the design reference for review, logs and completion behavior.

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

Integrations:

- Any future agent or editor integration is a thin adapter over an installed `tandem` CLI: use `execFile`/argument arrays, avoid shell interpolation, and consume CLI JSON.
- Do not duplicate Tandem protocol parsing or mutation behavior in an integration; call `tandem` and keep behavior in the CLI/protocol.

## Protocol architecture refactor campaign (completed)

The `refactor/protocol-architecture` campaign is **complete**. Campaign Epic
`task-146` is archived in `.tandem/logs/` and the campaign branch no longer
exists. There is no implementation freeze on `main`.

The campaign established five ownership boundaries that remain in force:
`protocol`, `project`, `app`, `cli`, and `tui`. `decision-8` records the
architecture decision; the campaign history is in the archived Logs.

One campaign practice survives as an ongoing repository rule:

- Delegate only independently reviewable Tasks, in isolated Task
  worktrees/branches, reviewed before integration.

The campaign also required human terminal validation for every visible TUI
change. That blanket requirement is retired; rule `always-12` governs when to
request review, based on whether you can verify the acceptance criteria rather
than on whether the work is visual.

## Validating TUI changes

A rendered terminal is evidence. Run the TUI in an hc terminal, wait for it to
draw, read the screen, then close it:

```sh
t=$(hc run --bg --no-notify --json --cwd "$SANDBOX" -- tandem tui | jq -r .address)
hc wait "$t" --output 'expected text' --timeout 10
hc read "$t"
hc kill "$t" && hc rm "$t"
```

`hc read` prints the exact screen as plain text (200x120 by default), which
settles layout, counts, and wrapping; drive keys with `hc keys "$t" f enter`.
It carries no ANSI styling, so colors and attributes need a human look. Build
release, not debug: debug builds render slowly enough to
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
