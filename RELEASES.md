# Tandem releases

Curated release notes for published Tandem versions. Add one meaningful `## X.Y.Z` section while preparing a release; `just release X.Y.Z` verifies that cargo-dist includes that section in the GitHub Release body. Detailed task, commit, and log history remains in Tandem.

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
