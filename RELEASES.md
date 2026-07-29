# Tandem releases

Curated release notes for published Tandem versions. Add one meaningful `## X.Y.Z` section while preparing a release; `just release X.Y.Z` verifies that cargo-dist includes that section in the GitHub Release body. Detailed task, commit, and log history remains in Tandem.

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
