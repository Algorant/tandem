# Tandem releases

Curated release notes for published Tandem versions. Add one meaningful `## X.Y.Z` section while preparing a release; `just release X.Y.Z` verifies that cargo-dist includes that section in the GitHub Release body. Detailed task, commit, and log history remains in Tandem.

## 0.12.4

Tandem v0.12.4 makes assignment workflows usable natively, with complete assignment reads, TUI lifecycle actions, and Git checkpoints at work boundaries.

### Added

- `tandem assignment <task-id> --json` returns complete Task and milestone definitions, blocker details, and an opaque scope token that stays stable during ordinary progress.
- The TUI's `a` Actions menu supports claim, deliver, block, resume, and completion. Task details show acceptance, blockers, milestone progress, and final evidence, with distinct completed, canceled, and failed outcomes.
- Native Git checkpoints commit only the owning `.tandem` path at assignment lifecycle boundaries. Milestone and Epic updates persist immediately but batch until an assignment boundary. Checkpoints never amend existing commits and preserve unrelated staged and working-tree changes.

### Fixed

- Delivery rejects missing or blank evidence before writing records or events. TUI evidence preserves commas in prose and retains entered values after errors.
- Git hooks can read Tandem without deadlocking. Checkpoint failures are reported separately from successful record writes, with the distinction visible even in narrow TUI footers.

### Changed

- `just dev` now builds release mode and opens a fresh Git-backed sandbox; `just dev-project` explicitly targets real project records. `just dev-check` runs tests and a native workflow smoke check.
- Integrations must not run their own checkpoint automation alongside the new native path. Lifecycle JSON results expose `recordWritten` and `checkpoint`; after a checkpoint failure, do not replay the successful lifecycle action.

## 0.12.3

Tandem v0.12.3 fixes two silent-zero defects: the TUI Decisions view now loads decision documents, and workspaces with pre-cutover embedded rules get a visible diagnostic instead of an empty list.

### Fixed

- The TUI Decisions tab lists every document in `.tandem/decisions/`, matching `tandem list --type decision`. Reload only read board documents, so the tab always showed zero. The reload fingerprint now covers the decisions directory, so decisions created or edited outside the running TUI appear after the reload interval. Board filtering is unchanged.
- `tandem rules list` and the TUI Rules view warn when `tandem.md` still carries a populated `rules:` block. The warning names the legacy location and states plainly that those rules are not active, instead of reporting zero rules silently. The same warning is present in the `--json` envelope. Embedded rules are not auto-migrated; move them to individual files under `.tandem/rules/`.

## 0.12.2

Tandem v0.12.2 repairs the accord and read paths left incomplete by the 0.12.0 cutover. Acceptance criteria no longer disappear when work starts, `show` returns a usable record again, and the accord block has one shape.

### Fixed

- Acceptance criteria survive the accord lifecycle. `claim` and every later transition rewrote the accord block without `accord.acceptance`, silently destroying the criteria a Task was created with. Escalation to validation then had no criterion to reference.
- `update --acceptance`, `--constraint`, and `--validation` write their values. The flags were parsed and discarded while the CLI reported success.
- `update` reports the fields it actually changed, or that nothing changed. It previously printed `Updated <id>` even when it wrote nothing, contradicting its own JSON `changes` array.
- `show --json` returns the full record: frontmatter, body, the whole accord including acceptance criteria, parent and children, validation, resolution, decision metadata, and `location`. It previously returned only `id`, `type`, and `title`, leaving no read path for machine consumers.
- `list --assignee` matches claimed work. Claim wrote `accord.assignee` while the filter read the top-level `assignee`, so filtering by assignee returned nothing.
- Document IDs sort numerically. `task-10` sorted between `task-1` and `task-2` everywhere IDs were ordered: the Board, `list`, `search`, logs, decisions, and the web read API.

### Changed

- Ownership is the top-level `assignee` only, as specified. `claim` sets it, `release` clears it, and `accord.assignee` is removed.
- `accord.validation` is a flat list of strings, matching `acceptance` and `constraints`. The nested `validation.commands` form and the `accord.validations` spelling are removed. Creation and transitions now render the accord block through one code path, so the shape cannot drift.
- The TUI no longer offers "Request human validation" on an active task. Escalation is an agent asking a human for judgment, and `tandem review <id> --criterion --note` is that path. Accept and archive and Request changes on delivered work are unchanged.
- Human `show` output is a short identity-and-status block: ID, type, title, location, state, accord, assignee. The TUI remains the human read surface.

### Compatibility

- Tasks claimed under 0.12.0 or 0.12.1 have no acceptance criteria left in the file. Restore them with `tandem update <id> --acceptance <text>`, which now works.
- Records written before this release carry `accord.assignee` and, after any transition, `validation.commands`. Neither is read anymore. Active Tasks recover their assignee on the next `claim`; archived Logs will display none. Re-supply planned validation with `tandem update <id> --validation <text>` where it matters.

## 0.12.1

Tandem v0.12.1 fixes two storage defects found while migrating a workspace to the protocol 0.3.0 cutover.

### Fixed

- `tandem add decision` now writes decision documents to `.tandem/decisions/` instead of `.tandem/tasks/`, and no longer writes a manual `date` field (dates are automatic; `decidedAt` is set when a Decision reaches accepted or rejected).
- Rules are now stored one file per rule under `.tandem/rules/` with composite ids such as `always-12`. `rules add|edit|delete` operate on the per-file store, `rules edit` supports `--clear source`, and `rules list` prints composite ids and filters by category. The TUI Rules view and web rules API read the same files.

### Compatibility

- Rules created with 0.12.0 live in the workspace config (`tandem.md`) with flat numeric ids. Workspaces that added rules under 0.12.0 should recreate them with `tandem rules add` so they land in the new per-file store, then remove the `rules:` block from `tandem.md`.
- Deadline-adjacent `tandem update` for decision documents is unchanged in this release.

## 0.12.0

Tandem v0.12.0 is the protocol 0.3.0 cutover: a rewritten clap-derived CLI, a new storage layout, and mandatory work agreements. The full contract is recorded in `decision-12`. This is a **breaking** release — read Compatibility before upgrading.

### Protocol

- Protocol version `0.3.0` with a new workspace layout: `.tandem/tasks/`, `decisions/`, `rules/`, `logs/`, and per-actor `events/`. "Board" is now a UI concept, not a directory.
- Every active Task carries a mandatory Accord with at least one acceptance criterion. `claim` assigns; `deliver` requires a summary and evidence; `complete` accepts a delivered Accord and archives atomically; `fail` archives as failed; `release` returns work to `ready`; `resume` unblocks.
- Task state is `todo`, `in-progress`, `validation`. Validation is entered only through explicit human escalation (`review <id> --criterion --note`); the `review.status` field is removed.
- Papercuts are low-priority Tasks tagged `papercut`, with a dedicated TUI section and tag filtering.
- Archived Logs preserve the full Task and Accord plus minimal `resolution{outcome,note,reviewer}` metadata; delivery evidence is not duplicated.
- Rules live one per Markdown file in `.tandem/rules/` with composite IDs (`always-12`). Decisions remain first-class ADR records with automatic dates.
- Every durable mutation appends a structured per-actor event.

### CLI

- The handwritten parser is replaced by a clap-derived model: 23 invocable commands, generated help on all 27 root/family/leaf surfaces, and global `-h/--help`, `-V/--version`, `-j/--json`. Every removed command and flag now fails with usage.
- Typed `add task|decision`; unified `show`/`list`/`search`/`update` across Tasks, Decisions, and Rules with `--scope active|archived|all`.
- `update` is deterministic: repeated flags replace the whole list, absent means unchanged, `--clear <field>` removes a value. Prose accepts leading hyphens; repeated scalar options are usage errors.
- Accord lifecycle `claim|deliver|rework|block|resume|release|fail`; `review` escalates to exceptional human validation; `complete` accepts and archives; `cancel`/`fail` archive with reasons.
- One global JSON envelope for success and failure with stable error codes; human results on stdout, warnings/errors on stderr; exit codes 0/1/2.
- Landing page restored to a grouped, descriptive command reference.

### TUI

- Four top-level views (Board, Logs, Rules, Decisions) preserved; Board gains a fourth section, Papercuts, derived from tagged Tasks and navigable by keyboard and mouse like the others.
- Quick-add and direct state movement are removed; Validation is adapted to exceptional review, accept→archive, and request-changes.
- Themes, mouse, hot reload, `$EDITOR`, and the State/Epic arrangement stay.

### Web

- The read-only web interface serves the 0.3.0 read models (resolution + validation surfaces; Papercuts as tagged Tasks).

### Fixed

- `<command> --help` now works on every surface — previously only 2 of 46 (papercut-1).
- Metadata lists can now be replaced and cleared via deterministic update + `--clear` (papercut-9).
- The TUI Papercuts section was not Tab-reachable or clickable; it is now a full peer section (papercut-10).
- The landing page had collapsed to four bare lines; grouped descriptions restored (papercut-11).
- Retained Add/quick-add and direct-Move actions from the previous TUI are fully removed (papercut-12).

### Compatibility

- **Breaking:** the new binary supports protocol `0.3.0` workspaces only. There is no `upgrade`/`migrate` command, converter, backup, or compatibility reader; older workspaces fail clearly. Migrating an existing workspace's coordination data is project-owner work, handled workspace by workspace.
- Removed commands: `upgrade`, `move`, `log list|show|search`, `decision list|show|update|withdraw`, `papercut add|list|show|resolve`, `version`, and `accord accept`.
- Removed flags: `--description` (now `--body`), `update --state` / `--parent-id` / `--parentId`, per-command `--json` (now global), and the Decision prose flags `--context`/`--consequence`/`--alternative`/`--date`.
- Scripts and integrations need updating to the new grammar and JSON envelope; the Pi adapter overhaul is tracked separately.

## 0.11.0

Tandem v0.11.0 changes what `validation` means. It is no longer where delivered work waits by default; it now marks work that a human was explicitly asked to look at.

### Protocol

- Accord actions no longer move workflow state, with two exceptions: `claim` moves `todo` to `in-progress`, and `rework` on a task in `validation` returns it to `in-progress` and clears the pending review. `deliver`, `accept`, `block`, and `fail` leave workflow state untouched.
- `review.status: pending` is the only entrance to `validation`. Adding a task directly in `validation`, or moving one there without a pending review, is rejected as `E067`.
- A task may remain in `validation` with an accepted review while awaiting completion.
- Completing a task with `review.status: pending` is now an error. Completing one with no review is silent; the previous `W020` warning is removed.
- Review requests are independent of accord status and are limited to the delegated Task boundary. Subtasks cannot be reviewed; Epics are exempt.
- `review.status: not-ready` is readable but is no longer produced by any command, matching how `accord: ready` is already treated.

### CLI

- New `tandem review request|accept|changes|reject <id>`. `request` sets `review.status: pending` and enters `validation`. `changes` and `reject` both resolve the review and return the task to `in-progress`, differing in recorded status and meaning rather than mechanics.

### TUI

- Board rows distinguish delivered work awaiting triage from work still in progress. A row in `validation` shows its pending review rather than a redundant delivered accord.
- A `Delivered · untriaged` Board filter lists work that has been delivered but not yet triaged.
- Draws are wrapped in synchronized output, removing tearing during redraw.

### Fixed

- A tracked `.tandem/actor-id` is now rejected with an actionable error naming `git rm --cached`. Committing that file gave every clone one identity, so parallel machines allocated the same event sequence numbers and collided at merge time. `git check-ignore` alone could not detect this, because ignore rules never apply to tracked files.
- After writing its ignore pattern, Tandem verifies the pattern actually took effect. A repository `.gitignore` negation can override `.git/info/exclude`, which previously went unnoticed.

### Upgrading

Existing workspaces keep working, but boards built under the previous behavior may hold tasks sitting in `validation` without a pending review. Those tasks are readable and completable; they simply no longer match how work reaches `validation`. Move them back to `in-progress`, or request a review if one is genuinely wanted.

Scripts that relied on `tandem accord deliver` or `accept` moving a task into `validation` need updating. Use `tandem review request` to escalate work to a human, and `tandem complete` directly for objective work.

## 0.10.3

Tandem v0.10.3 restores fast loading for the local read-only web interface on established workspaces.

### Fixed

- Web reference validation now reuses the Board and Log hierarchy already loaded in memory instead of rereading and reparsing every document for every loose reference.
- References to Papercuts still resolve through canonical Papercut filenames without parsing their contents, so malformed Papercuts remain isolated from unrelated Board and web reads.
- Project-scale `/api/v1/project` responses that previously exceeded 30 seconds returned in about 120–161 ms during release validation on the Tandem and Pi workspaces.

## 0.10.2

Tandem v0.10.2 stops active work from being stranded under archived parents.

### Protocol

- Completing or archiving a task now requires every descendant to be resolved first. This covers both an Epic with active child Tasks and a Task with active Subtasks, and matches the behavior `tandem cancel` already had.
- Creating a task under a parent that already lives in `.tandem/logs/` is rejected.

### Fixed

- `tandem complete` no longer archives a parent while its Subtasks stay active on the Board. Those children previously remained on the Board with no warning and no reachable parent, and could contradict the parent's own completion summary. Documents already in that state remain completable and cancelable, so existing workspaces stay repairable.

## 0.10.1

Tandem v0.10.1 makes Papercuts visible during terminal work and improves the consistency and responsiveness of the TUI.

### TUI

- A compact global `Papercuts N` indicator opens a read-only Papercuts list and detail panel without adding another main view or Board state.
- A coherent fixed input model replaces conflicting case-sensitive shortcuts with explicit Board filter, move, and Validation pickers.
- The universal `?` reference groups global, navigation, view, dialog, utility, and mouse controls and is available from every non-text surface.
- Board chrome uses one concise command footer, avoids repeated state and row counts, and uses `Enter` as the single row activation key.

### Fixed

- The Logs view no longer continuously redraws and rebuilds every archived row while idle. Event-driven rendering and visible-viewport projection reduce idle CPU from the previous project-scale growth to about 1% with 250 Logs in the release benchmark.
- Logs keyboard and mouse selection retain absolute filtered-result positions while only visible rows are projected.
- External-change reloads and transient status expiry continue to wake and redraw the TUI on their deadlines.

### Developer validation

- `just bench-tui-idle` generates temporary 10, 50, 100, and 250-Log workspaces and measures the real release TUI through a drained fixed-size PTY on Linux.
- Caller-supplied benchmark preview paths are never replaced or deleted when they already exist.

## 0.10.0

Tandem v0.10.0 introduces Papercuts: a lightweight project inbox for preserving small, non-blocking friction without interrupting active work.

### Features

- New `tandem papercut add|list|show|resolve` commands record and manage project-local Papercuts.
- Papercuts use lightweight Markdown records under `.tandem/papercuts/` with stable IDs, optional tags and references, free-form context, and resolution notes.
- Global `tandem search` finds Papercut titles, bodies, tags, references, statuses, and resolution notes.
- Tasks and Decisions can reference Papercuts without adding Papercuts to Tandem's document hierarchy.
- The bundled `pi-tandem` adapter exposes `tandem_papercut` for agent-driven capture, inspection, and resolution.

### Improvements

- Agent guidance distinguishes notable workflow friction from expected failures, blockers, planned work, Decisions, Rules, and telemetry.
- CLI, workspace, concepts, extensions, adapter, and reference documentation now explain the complete Papercut workflow.

### Compatibility

- Existing workspaces require no migration. `.tandem/papercuts/` is optional and created only when the first Papercut is added.
- Papercuts remain outside the Board, Logs, hierarchy, Accord, review, completion progress, and TUI.

## 0.9.0

Tandem v0.9.0 adds a polished, read-only web interface for viewing a local workspace in the browser.

### Web interface

- `tandem web` serves the current workspace on an automatically selected loopback port and opens it in the default browser; `--port` and `--no-open` support explicit local workflows.
- The browser includes Board and Validation views, task and relationship details, Logs with search, Rules, Decisions, and project health warnings.
- Semantic HTML, a small dependency-free JavaScript client, responsive Verdigris styling, keyboard navigation, reduced-motion support, and light and dark palettes keep the interface fast and accessible.
- Revision polling refreshes changed workspace data while preserving filters, focus, and scroll position.

### Security and architecture

- The server is read-only and loopback-only, validates the Host header, limits methods, targets, bodies, and concurrency, and sends restrictive browser security headers without permissive CORS.
- The embedded server and bundled frontend reuse Tandem's canonical application, project, and protocol layers. They require no database, Node runtime, remote assets, or separate frontend build at runtime.
- `just web` provides a one-command development shortcut from the repository checkout.

## 0.8.4

Tandem v0.8.4 makes the documentation easier to follow and adds common repository work badges to the Board.

### Documentation

- The documentation site now has clearer landing, quick-start, CLI, TUI, workspace, concepts, and extensions pages with a Verdigris-aligned visual treatment and navigation.
- New fully agentic and human-in-the-loop workflow guides show how to use Tandem across different review and delegation models.
- Framework-neutral agent guidance now explains coherent commit boundaries for durable `.tandem` workspace data, prudent local squashing, and shared-history safety.

### TUI and themes

- Board rows now render `BUG`, `FEAT`, and `CHORE` as minimal built-in work badges while project-specific tags remain opt-in.
- Theme-owned orange, sand/beige, and purple tones can be configured or overridden and work with every badge style and terminal no-color mode.
- Default Dark and Verdigris include distinct palettes for the new badges: corrective work uses orange, features use warm sand, and maintenance uses purple.

### Fixed

- Documentation navigation and internal links found during the site overhaul now resolve to the intended current pages.

## 0.8.3

Tandem v0.8.3 defines a framework-neutral contract for agents and integration adapters.

### Agents and adapters

- Universal guidance now explains workspace discovery, authority layers, lifecycle boundaries, context retrieval, and safe adapter behavior without depending on one agent framework.
- Rule categories now have explicit operational meanings: Always requires, Never prohibits, Prefer defines a justified default, and Context supplies non-directive information.
- Mixed directives have classification guidance so narrow conditions do not weaken requirements or prohibitions into Context.
- Adapter implementations remain separate from core Tandem work; future adapter changes use explicit implementation handoffs rather than framework-specific protocol guidance.

## 0.8.1

Tandem v0.8.1 makes project rules easier to scan and read in the TUI.

### Rules view

- Rules use a dense, stable one-line list with category-colored IDs, neutral previews, muted source metadata, and clear selection treatment.
- `Enter` toggles a full-width preview pane that follows keyboard and mouse selection and wraps the complete selected rule.
- The bordered list pane dynamically fits small categories, caps large categories near two-thirds with scrolling, and gives all remaining space to the preview.
- Always, Never, Prefer, and Context retain distinct green, red, amber, and purple visual identities.
- Short terminals preserve minimum list and preview space and safely fall back to the full list when both panes cannot fit.

## 0.8.0

Tandem v0.8.0 gives every writable checkout and linked worktree a stable, isolated event-writer identity without configuration.

### Events and collaboration

- Tandem atomically creates and reuses an ignored `.tandem/actor-id` UUID for each independent checkout or linked worktree.
- New audit events remain tracked in separate `.tandem/events/<actor-id>.jsonl` ledgers, preserving existing per-actor sequence identities and legacy event reads without migration.
- Concurrent processes in one worktree converge on the same identity and retain serialized event appends, while independent clones and worktrees use distinct ledgers that merge normally through Git.
- Actor identity is non-configurable, so shell variables and orchestration integrations cannot accidentally collapse independent worktrees onto one event ledger.
- Git projects add the identity path to local exclude state without changing tracked project policy; non-Git workspaces retain the same local identity behavior.

### Integration boundaries

- Tandem alone owns actor identity generation, persistence, validation, and event writing.
- Herdr, Worktrunk, Pi Workers, Reviewers, Subagents, and Pi-Tandem remain identity-unaware; retained or recovered worktrees reuse their identity and new worktrees receive a new one.

## 0.7.2

Tandem v0.7.2 gives the bare `tandem` command a polished, concise landing page.

### CLI

- Commands are grouped by purpose with aligned names and short descriptions.
- The landing page includes every top-level command and points to `tandem <command> --help` for detailed usage.
- Restrained terminal styling is enabled only for interactive terminals; piped output and `NO_COLOR` remain ANSI-free.

## 0.7.1

Tandem v0.7.1 fixes protocol 0.2 upgrades for projects containing recognized legacy priority aliases.

### Fixed

- Explicit `tandem upgrade` canonicalizes legacy `med` and `normal` priorities to `medium` in active documents and completed logs while preserving unrelated frontmatter and Markdown bodies.
- Already-canonical priorities and archived log content remain unchanged; ordinary commands still never upgrade or mutate projects implicitly.

## 0.7.0

Tandem v0.7.0 establishes protocol 0.2 and a canonical implementation architecture shared by peer CLI and TUI interfaces.

### Architecture

- Repository protocol Markdown remains normative, with one executable Rust protocol layer owning documents, IDs, hierarchy, workflow, accords, reviews, events, and diagnostics.
- `project::TandemProject` now owns concrete `.tandem` discovery, preservation, locking, atomic writes, archives, and event files.
- Shared application operations coordinate protocol semantics and project I/O for both CLI and TUI mutations.
- The CLI and TUI are explicit peer interfaces; `main.rs` and `tui/mod.rs` are focused wiring roots, and cohesive TUI modules own Board projection, rendering, input, reload, validation, chrome, and text.

### Protocol compatibility

- New projects use protocol 0.2.0.
- Existing protocol 0.1.0 projects require an explicit `tandem upgrade`; ordinary commands never upgrade project data implicitly.
- Legacy custom task-like documents remain preserved as deprecated read-only content after upgrade.

### Validation

- Added compiled-command behavior coverage for protocol compatibility, project mutations, hierarchy, completion, logs, rules, and decisions.
- Strict Clippy, extension smoke tests, documentation builds and link checks, packaging checks, PTY tests, and direct TUI validation cover the refactored boundaries.

## 0.6.5

Tandem v0.6.5 makes Board workflow-state chips themeable, with a Verdigris preset tuned for clear work-state scanning.

### TUI and themes

- Theme files can declare reusable color aliases and assign distinct colors to any configured workflow-state chip.
- The Verdigris preset renders WIP in burnt copper and validation in heather purple, while TODO keeps its subdued neutral fallback.

## 0.6.4

Tandem v0.6.4 removes the retired `ready` accord action from active interfaces while preserving compatibility for existing records.

### Fixed

- Bare `tandem accord` and Pi-Tandem now advertise only supported accord actions.
- `tandem accord ready` reports the current supported actions instead of implying that it remains available.
- Existing persisted `accord.status: ready` values remain readable for compatibility.

## 0.6.3

Tandem v0.6.3 simplifies accord claims and adds explicit correction paths for decision records.

### Added

- New work starts with `tandem accord claim`; legacy `ready` records remain readable.
- Decisions can be updated or withdrawn through supported CLI commands with audit history.

## 0.6.2

Tandem v0.6.2 improves the Board hierarchy and its release-facing guidance.

### Fixed

- Board hierarchy presentation now follows the canonical Epic, Task, and Subtask relationships.
- Release documentation better distinguishes the supported CLI and Pi integration workflows.

## 0.6.1

Tandem v0.6.1 strengthens task cancellation and safe task-body editing.

### Added

- Task cancellation records a reasoned archived outcome while preserving project history.
- Task bodies can be edited through the supported CLI workflow.

## 0.6.0

Tandem v0.6.0 establishes the canonical Epic, Task, and Subtask hierarchy across the protocol, CLI, TUI, and Pi integration.

### Added

- Direct Epic Tasks use global task IDs, while only direct Task children use parent-derived Subtask IDs.
- CLI and TUI hierarchy displays validate and expose the canonical relationships consistently.
