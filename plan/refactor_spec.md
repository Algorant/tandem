# Tandem Rust Architecture Refactor Specification

- Status: reviewed specification; ready for Task decomposition and architecture decision
- Date: 2026-07-22
- Inputs: task-145 modularization research, Tandem protocol specification, decision-7 hierarchy work, and follow-up architecture discussion

## Decision posture

This reviewed planning specification records the project owner's resolved architecture and compatibility choices. It is not itself the accepted architecture decision and does not authorize implementation.

Epic `task-146` records the campaign boundary. The remaining sequence is:

1. commit this reviewed specification;
2. decompose `task-146` into independently reviewable direct Tasks;
3. record one broad accepted Tandem architecture decision referencing the specification, Epic, and Tasks;
4. create the dedicated refactor integration branch;
5. begin a Task only after it is explicitly authorized.

The eventual decision should lock the architectural boundaries and dependency direction, not every final filename or line count.

## Purpose

Tandem's Rust implementation currently concentrates most behavior in `tandem/src/main.rs` and `tandem/src/tui.rs`. The code works and has broad test coverage, but protocol semantics, concrete Tandem-project file access, application operations, CLI presentation, and TUI behavior are not visibly separated.

The refactor should make the canonical Tandem protocol implementation unmistakable and shared while keeping the CLI and TUI as separate peer interfaces over the same behavior.

The campaign combines an explicitly approved protocol `0.2.0` compatibility change with an organizational refactor. The protocol change must land in dedicated behavior-changing Tasks and commits. Move-only stages must not change protocol behavior, CLI output, persisted data, events, TUI behavior, keybindings, themes, or release packaging.

## Goals

- Make the normative Markdown protocol and its executable Rust implementation visibly distinct.
- Establish one canonical Rust protocol layer for document meaning, IDs, hierarchy, lifecycle, accords, review, events, and validation.
- Replace the vague implementation name `workspace` with a concrete `project` module representing one discovered Tandem project and its `.tandem/` files.
- Retain a shared application layer so CLI and TUI do not independently orchestrate the same mutations.
- Treat CLI and TUI as peer interfaces rather than successive layers.
- Move the TUI root to `tui/mod.rs` so feature adapters and helpers have a conventional module home.
- Preserve one Cargo package and one production binary crate.
- Perform the work on a dedicated integration branch through small, attributable checkpoints.
- Isolate existing lint debt explicitly, remove it as modules are completed, and finish with strict Clippy producing zero warnings.
- Add simple tests that run the real `tandem` command so internal moves cannot silently change user-visible behavior.
- Implement the approved protocol `0.2.0` policy separately from behavior-preserving module movement.
- Update project documentation and agent guidance as part of the campaign.

## Non-goals

- No root Cargo workspace.
- No `crates/` directory or additional package.
- No `lib.rs` or supported Rust library API during this campaign.
- No protocol redesign hidden inside module movement.
- No new database or opaque project-data layer.
- No generic repository, filesystem, renderer, command, or component traits without a demonstrated second implementation.
- No dependency-injection container, plugin framework, event bus, or async-runtime migration.
- No replacement of manual CLI parsing or hand-built JSON merely to facilitate movement.
- No broad Ratatui component-framework or Elm/Redux rewrite.
- No removal of legacy behavior, including `review.rs`, unless separately approved.
- No code-size reduction target that encourages artificial abstractions.

## Terminology

### Normative protocol specification

The Markdown documents under repository `protocol/` are the source of truth for Tandem's format and semantics. They describe what conforming implementations must do.

### Executable protocol

`tandem/src/protocol/` is the Rust implementation of the normative protocol. Its module documentation should link back to the normative specification and state that it is an implementation rather than a second specification.

### Tandem project root

The project directory containing the discovered Tandem data directory. Discovery currently centers on `.tandem/tandem.md`, with the documented root `tandem.md` compatibility path where applicable.

### Tandem data directory

The concrete project-local `<project-root>/.tandem/` directory containing configuration, active documents, logs, events, and tool sidecars.

### Project module

`project` is the concrete filesystem-facing module for one discovered Tandem project. It finds the project root, resolves the project-local `.tandem/` directory, reads and preserves documents, locks snapshots, detects conflicts, and writes files and events atomically.

The primary concrete type is `project::TandemProject`. It contains or exposes the project root and the resolved `.tandem/` paths needed by the implementation. It is not a generic repository, storage interface, or database abstraction.

### Application layer

`app` coordinates complete use cases such as add, move, update, complete, accord transitions, Rules mutation, and Decision creation. It combines protocol rules with concrete `TandemProject` operations and returns typed outcomes without printing or rendering.

### Interfaces

The CLI and TUI are two peer human/tool interfaces:

- `cli` owns command syntax and textual/JSON presentation.
- `tui` owns terminal interaction, transient state, projection, and rendering.

The external `pi-tandem` extension remains another adapter, but it continues to call the installed CLI rather than importing or reimplementing Tandem protocol behavior.

## Proposed dependency direction

```text
repository protocol/*.md
        |
        | normative requirements
        v
+-----------------------+
| executable protocol   |
| src/protocol/         |
+-----------------------+
       ^           ^
       |           |
+-------------+  +----------------+
| project     |  | application    |
| .tandem I/O |  | use cases      |
+-------------+  +----------------+
       ^             ^         ^
       |             |         |
       +-------------+---------+
                     |
             +-------+-------+
             |               |
          +-----+          +-----+
          | CLI |          | TUI |
          +-----+          +-----+
```

In source dependency terms:

```text
protocol      -> standard library and protocol-focused dependencies only
project       -> protocol
app           -> protocol + project
cli           -> app + protocol result/value types
tui           -> app + protocol result/value types
main          -> cli and TUI startup/composition
```

The diagram is not intended to suggest that `project` calls application code. Application code directs concrete `TandemProject` operations. Both depend on protocol types and rules.

### Dependency rules

- `protocol` must not import `project`, `app`, `cli`, or `tui`.
- `project` must not infer Epic/Task/Subtask roles or duplicate protocol validation.
- `app` must not print, render Ratatui widgets, or parse process arguments.
- `cli` and `tui` must not implement hierarchy, ID, lifecycle, accord, review, or event rules independently.
- CLI and TUI durable mutations must go through shared `app` use cases.
- TUI Board projection may query canonical protocol hierarchy results, but it may only arrange those results for display.
- Rendering must not write Tandem files.
- `project` must preserve unknown fields and Markdown bodies according to protocol requirements.
- Cross-layer APIs should use private items by default, `pub(super)` where possible, and narrow `pub(crate)` only for genuine sibling-layer seams.

## Proposed source structure

This is a target map. A file should be created only when cohesive code moves into it.

```text
tandem/src/
├── main.rs                         # process entry, composition, exit handling
├── protocol/
│   ├── mod.rs                      # executable-protocol boundary and exports
│   ├── config.rs                   # logical project configuration values
│   ├── document.rs                 # document values and metadata accessors
│   ├── ids.rs                      # ID parsing and role-specific forms
│   ├── hierarchy.rs                # sole role/relationship graph authority
│   ├── workflow.rs                 # workflow and completion semantics
│   ├── accord.rs                   # accord values and transitions
│   ├── review.rs                   # review values and validation
│   ├── event.rs                    # canonical event envelope and names
│   └── diagnostic.rs               # structural diagnostics and severity
├── project/
│   ├── mod.rs                      # concrete TandemProject boundary
│   ├── discovery.rs                # project-root and .tandem discovery
│   ├── frontmatter.rs              # raw-source preservation and minimal patches
│   ├── read.rs                     # strict and tolerant document/event reads
│   ├── write.rs                    # locks, snapshots, conflict checks, atomic writes
│   └── events.rs                   # per-actor JSONL file operations
├── app/
│   ├── mod.rs                      # shared queries, snapshots, and outcomes
│   ├── tasks.rs                    # add, move, update, complete
│   ├── accord.rs                   # accord and Validation operations
│   ├── decisions.rs                # Decision operations
│   └── rules.rs                    # Rules operations
├── cli/
│   ├── mod.rs                      # dispatch, help, version
│   ├── args.rs                     # manual parsers and option records
│   ├── commands.rs                 # thin adapters into app
│   └── output.rs                   # exact human and JSON output
└── tui/
    ├── mod.rs                      # TuiApp aggregate and event-loop wiring
    ├── terminal.rs                 # terminal enter/restore/suspend/resume safety
    ├── editor.rs                   # editor selection, parsing, and execution
    ├── reload.rs                   # reload, fingerprints, selection restoration
    ├── input.rs                    # keyboard/mouse translation and action dispatch
    ├── validation.rs               # Validation prompts and UI adapter behavior
    ├── chrome.rs                   # tabs, header, footer, help, hit geometry
    ├── text.rs                     # shared Markdown/detail/wrapping helpers
    ├── board/
    │   ├── mod.rs                  # Board state and feature dispatch
    │   ├── projection.rs           # canonical-protocol-backed view models/filtering
    │   └── render.rs               # rows, details, previews, layout, hit regions
    ├── decisions.rs                # feature-local state/input/rendering
    ├── logs.rs                     # feature-local loading/projection/rendering
    ├── review.rs                   # retained until a separate product decision
    ├── rules.rs                    # feature-local state/input/rendering
    └── theme.rs                    # theme/config/style ownership
```

`src/tui.rs` should become `src/tui/mod.rs` in one dedicated behavior-preserving move. Rust must never have both paths defining the same `tui` module.

## Protocol ownership

"Canonical" does not mean "can never change." It means a change requires explicit protocol review and, where compatibility changes, a protocol decision/version policy. CLI and TUI cannot quietly redefine it.

### Canonical protocol concerns

| Concern | Proposed classification | Explanation |
| --- | --- | --- |
| Protocol version field and compatibility check | canonical | The target protocol is `0.2.0`; `protocolVersion` identifies the format contract and gates ordinary project operations. |
| `.tandem/`, `board/`, `logs/`, and per-actor `events/` roles | canonical | These locations are part of the on-disk Tandem protocol, even though `project` resolves their concrete paths. |
| Markdown documents with YAML frontmatter | canonical | The protocol defines how the representation is interpreted; `project` reads it and preserves or minimally patches the raw source. |
| Required document identity and type fields | canonical | IDs, document types, and required fields must mean the same thing in every adapter. |
| Unknown-field and Markdown-body preservation | canonical | A compliant mutation must not destroy information it does not understand. |
| Epic, Task, and Subtask derivation | canonical | Roles derive from resolved documents and never from ID shape alone. |
| Parent relationships | canonical | `epic-task`, `subtask`, and generic `parent` have one meaning. |
| Role-specific ID forms and allocation without reuse | canonical | Epics/Tasks use global IDs; Subtasks use parent-derived IDs; Board and Logs participate in allocation. |
| Structural hierarchy failures | canonical | Parented Epics, children beneath Subtasks, role/ID mismatches, and invalid reparenting are errors. |
| Workflow field name | canonical | Task workflow is stored in `state`; configuration chooses allowed active values. |
| Completion/archive behavior | canonical | Completion moves active work to Logs rather than creating a persistent completed column by default. |
| Missing review/accord acceptance | canonical | Tandem always warns but still allows completion; structural protocol errors continue to block. |
| Accord field and status vocabulary | canonical | Accord state is separate from workflow and uses the accepted status set. |
| Review field and status vocabulary | canonical | Review metadata is separate from workflow and accord state. |
| Decision status semantics | canonical | ADR-style Decision status is metadata, not task workflow state. |
| Event required envelope and identity | canonical | Required fields, actor ownership, and `<actor>:<seq>` identity must be consistent. |
| Structural-error versus warning policy | canonical | Core structure and references fail closed; related-reference and policy findings may warn. |
| Supported document types | canonical | `task` and `decision` are the only supported first-class document types. Existing custom-type data may be inspected as legacy read-only content; Tandem does not create new custom types or custom-type documents. |
| Priority values | canonical | Optional `priority`, when present, is one of `low`, `medium`, `high`, or `critical`. |
| Effort values | canonical | Optional `effort`, when present, is one of `trivial`, `small`, `medium`, or `large`. |
| Legacy project completion settings | canonical compatibility | Existing settings are preserved and reported as deprecated, but they cannot override canonical warn-but-complete behavior. |

### Project-configurable concerns

| Concern | Proposed classification | Explanation |
| --- | --- | --- |
| Project title | configurable | It names one Tandem project and has no cross-project canonical value. |
| Active workflow states | configurable with canonical defaults | `todo`, `in-progress`, and `validation` are defaults; projects may define additional active states. The field remains `state`. |
| Project rules | configurable data | Rule object shape and category names are canonical; rule text and IDs belong to the project. |
| Agent instructions | configurable data | The protocol preserves them, while humans and adapters decide how to consume them. |
| Tags, assignees, references, and related paths | configurable document data | Their structure is validated, but their actual values are project-specific. |
| Actor identity and display name | configurable identity data | Event ownership rules are canonical; actor IDs and cosmetic names differ by writer. |

### Interface-only concerns

| Concern | Owner | Explanation |
| --- | --- | --- |
| Theme, colors, badges, and layout | TUI | These do not alter protocol meaning. Protocol files may preserve selectors without interpreting presentation. |
| Keyboard and mouse bindings | TUI | They are interface policy, not protocol semantics. |
| Human table formatting and help text | CLI | They are stable user contracts but not protocol data rules. |
| JSON response envelopes | CLI | They are CLI API contracts derived from protocol/app outcomes, not the on-disk protocol itself. |
| Pi tool names and argument schemas | pi-tandem | The extension remains a thin adapter over CLI behavior. |

## Protocol 0.2 compatibility policy

The compatibility policy is resolved:

- New Tandem projects use `protocolVersion: 0.2.0`.
- When Tandem discovers a `0.1.0` project, it refuses every project operation except an explicit `tandem upgrade`; upgrading is never implicit on the first write. Process-level help and version output do not open a project and remain available.
- The detailed conversion mechanics of `tandem upgrade` are deferred to the dedicated Task that implements that command. This architecture specification does not prescribe automatic document conversion.
- `task` and `decision` are the only document types Tandem creates or treats as first-class protocol documents.
- Existing custom-type declarations and documents are preserved as legacy read-only content and produce a deprecation warning. They may be listed, shown, and searched after upgrade, but Tandem does not create new custom declarations/documents or apply mutations to legacy custom documents.
- Existing project-level completion-policy settings are preserved and produce a deprecation warning, but their values are ignored. Tandem always warns when review or accord acceptance is missing and still allows completion unless a structural error blocks it.
- Priority is optional and, when present, must be `low`, `medium`, `high`, or `critical`.
- Effort is optional and, when present, must be `trivial`, `small`, `medium`, or `large`.

These protocol changes require explicit normative-document, implementation, diagnostic, and behavior-test updates. They must land in behavior-changing commits before or separately from move-only module extraction.

## Protocol and project boundary

The protocol owns logical meaning. `project` owns concrete `.tandem` files and filesystem safety.

Examples:

- `protocol::document` defines how a Tandem `Document` is interpreted and validated.
- `project::frontmatter` reads Markdown/YAML and retains raw source needed for byte-preserving minimal patches.
- `protocol::hierarchy` validates resolved documents.
- `project::read` locates and reads the files supplied to that validator.
- `protocol::event` defines the event envelope and accepted names.
- `project::events` finds the actor log, verifies sequence rules, and appends JSONL safely.
- `protocol::workflow` defines completion semantics.
- `project::write` performs the snapshot, lock, conflict check, atomic write, and board-to-log movement requested by `app`.

Raw patching remains a `project` concern because it operates on source bytes. Whether a field is valid or what it means remains a protocol concern.

## Application boundary

`app` remains explicit.

A typical mutation should read as:

```text
CLI/TUI input
  -> app use-case input
  -> TandemProject loads a coherent project snapshot
  -> protocol validates and derives canonical meaning
  -> app chooses the operation and outcome
  -> TandemProject writes atomically and appends events
  -> app returns typed result and warnings
  -> CLI prints or TUI reloads/renders
```

`app` should use concrete functions and outcome types. It should not begin with service traits or abstract ports merely to imitate a framework. A trait becomes justified only when Tandem has a real second implementation boundary.

## Interface boundaries

### CLI

CLI owns:

- process arguments and manual parsing;
- usage and help text;
- dispatch into application operations;
- stdout/stderr choice and exit categories;
- compact human tables/details;
- exact JSON response envelopes.

CLI does not own protocol validation or filesystem mutation. Parsing the `tui` command may return a TUI-startup request to `main`; CLI must not import or call TUI implementation code.

### TUI

TUI owns:

- terminal lifecycle;
- transient view, focus, selection, modal, filter, and scroll state;
- keyboard and mouse translation;
- canonical-result projection into rows and details;
- Ratatui rendering and frame-local hit regions;
- human-visible status and diagnostics.

TUI does not own protocol inference or direct durable mutation orchestration. It invokes `app`, then reloads canonical state.

## Dedicated integration-branch strategy

The refactor should develop on a long-lived integration branch rather than partially landing an incoherent architecture on `main`.

Proposed integration branch:

```text
refactor/protocol-architecture
```

Branch rules:

1. Create it from a clean `main` after this specification, the architecture decision, and initial Epic/Task records are accepted.
2. Keep every stage as a small, reviewable commit or short commit series.
3. Delegated work uses isolated worktrees/branches and is reviewed before being integrated into the refactor branch.
4. Freeze Rust CLI/TUI implementation work on `main` while the refactor branch is active.
5. A critical Rust fix that cannot wait must land on `main`, then be integrated immediately into the refactor branch and pass the full checkpoint suite.
6. Documentation, planning, extensions, and unrelated non-Rust work may continue on `main` when they do not alter the frozen Rust architecture.
7. Synchronize eligible `main` changes into the refactor branch at explicit module checkpoints rather than waiting until the end.
8. Do not merge unfinished module boundaries back to `main` merely to reduce branch age.
9. Preserve move-only commits separately from later API cleanup where practical so regressions remain attributable.
10. Merge the completed branch to `main` only after final architecture, behavior, lint, documentation, and human TUI validation pass.
11. Do not publish a release from the integration branch unless separately requested.

Freezing Rust implementation work substantially reduces divergence risk. Explicit synchronization remains necessary for critical fixes and eligible non-Rust changes.

## Lint and warning strategy

The campaign must not silently inherit existing warnings as permanent debt.

### Initial isolation

1. Run strict Clippy and record every current project diagnostic.
2. For each existing diagnostic that cannot be safely fixed in the same small preparation change, add the narrowest temporary `#[expect(clippy::...)]` annotation at the affected item, with a reason naming the refactor checkpoint that must remove it.
3. Prefer `#[expect]` over `#[allow]`: an expectation itself becomes visible when the expected lint no longer occurs.
4. Do not use `#![allow(warnings)]`, a crate-wide Clippy category allowance, or a broad module allowance that would hide newly introduced warnings.
5. After this isolation change, `cargo clippy --all-targets -- -D warnings` must pass on the integration branch.

Example shape:

```rust
#[expect(
    clippy::too_many_arguments,
    reason = "legacy seam; remove when Board rendering is extracted"
)]
fn legacy_board_helper(/* existing arguments */) {
    // Existing behavior.
}
```

### Module checkpoint

A module is not complete until:

- warnings in the moved/touched code are fixed rather than newly suppressed;
- temporary expectations assigned to that module are removed;
- no new expectation/allow annotation is introduced without explicit review;
- formatting, tests, command-level behavior tests, and strict Clippy pass;
- dependency direction and visibility rules pass review.

### Final checkpoint

Before merging to `main`:

- strict Clippy passes with zero diagnostics;
- every temporary refactor `#[expect]`/`#[allow]` is removed;
- any pre-existing intentional suppression is separately reviewed and documented rather than hidden in the refactor baseline;
- no module inherits a lint exception merely because its code originated in `main.rs` or `tui.rs`.

## Tests that run the real command

The earlier research called these "black-box integration tests." In simpler terms, they are tests that use Tandem the way a person or shell script does.

For example, a test should:

1. build the `tandem` executable;
2. create a temporary project directory;
3. run commands such as `tandem init`, `tandem add`, `tandem list --json`, or an invalid command;
4. inspect the exit status, stdout, stderr, and resulting `.tandem/` files;
5. remove the temporary directory.

Proposed path:

```text
tandem/tests/cli_behavior.rs
```

These tests add a test harness, not another production package, library API, or executable. They protect behavior that internal unit tests can miss when code moves between modules.

Required command-level coverage should include:

- help and version;
- unknown command and unknown flag;
- missing project and missing document;
- representative add, move, update, complete, and accord flows;
- representative human-readable and JSON reads;
- canonical hierarchy success and failure;
- preservation of unknown frontmatter/body content;
- event and completed-log effects;
- creation of protocol `0.2.0` projects and refusal of ordinary operations on `0.1.0` projects;
- legacy custom-type preservation/read-only warnings and rejection of new custom-type creation;
- preservation and deprecation of legacy completion settings while canonical completion warnings remain active.

Tests should create their own temporary project data and should not add a permanent fixture directory.

TUI protection remains a combination of focused unit/TestBackend tests, PTY behavior checks for terminal lifecycle, and human `just dev` validation at stages that can affect visible behavior.

## Common module checkpoint

Every implementation Task that completes a module or coherent seam should provide:

1. a clean, reviewable diff with movement separate from redesign;
2. explicit source and target ownership;
3. no prohibited reverse dependency;
4. no duplicated protocol rule;
5. no changed CLI/TUI behavior unless separately approved;
6. colocated unit tests moved with implementation;
7. real-command behavior tests where a process boundary is affected;
8. `cargo fmt --check` passing;
9. `cargo test` passing with no unexplained test-count reduction;
10. `cargo clippy --all-targets -- -D warnings` passing under the temporary-expectation policy;
11. module-specific lint expectations removed;
12. clean `git status --short` and one focused commit or justified commit series;
13. documentation updates for any stabilized ownership boundary.

Human terminal validation is additionally required for visible TUI stages.

## Staged implementation plan

The Epic should decompose these stages into normal independently managed Tasks. Stages describe order and dependency; they are not permission to start automatically.

### Stage 0 — architecture, compatibility, branch, and behavior guardrails

- Use existing Epic `task-146` and create the independently managed direct Task decomposition.
- Record the broad accepted architecture decision.
- Update initial project and agent guidance to name the new boundaries.
- Create `refactor/protocol-architecture` from the approved base.
- Add real-command CLI behavior tests.
- Implement the approved protocol `0.2.0` compatibility policy in dedicated behavior-changing Tasks and commits, including normative protocol updates and the explicit upgrade gate.
- Keep detailed `tandem upgrade` conversion mechanics scoped to the Task that implements that command rather than expanding this architecture document.
- Isolate every existing Clippy diagnostic with narrow temporary expectations or safe direct fixes.
- Require strict Clippy to pass before structural movement begins.

### Stage 1 — establish the TUI module directory on low-risk leaves

- Move `src/tui.rs` to `src/tui/mod.rs` without behavior changes.
- Extract the editor seam to `tui/editor.rs` with its tests.
- Extract terminal enter/restore/suspend/resume behavior to `tui/terminal.rs`.
- Run full tests and PTY/editor lifecycle validation.

This stage proves module, visibility, testing, and lint conventions before canonical protocol code moves.

### Stage 2 — extract the executable protocol

Move canonical semantics as independently reviewable units:

1. protocol document/config values;
2. ID parsing and allocation rules;
3. hierarchy roles, relationships, and structural validation as one semantic authority;
4. workflow/completion semantics;
5. accord and review values/transitions;
6. event envelope and diagnostic categories.

Move tests with each unit. Do not alter decision-7 behavior, accepted statuses, error severity, or persisted shapes in a movement commit.

### Stage 3 — establish precise project ownership

- Introduce `project::TandemProject` as the concrete representation of one discovered Tandem project.
- Extract project-root and `.tandem/` discovery.
- Extract frontmatter/raw-source preservation and minimal patches.
- Extract strict/tolerant reads.
- Extract hierarchy locking, snapshots, conflict checks, sequential creation, and atomic writes.
- Extract per-actor event file operations.

`project` must consume protocol values and never infer roles itself.

### Stage 4 — unify application operations

Move one mutation family per review and switch both CLI and TUI callers before proceeding:

1. Task add/move/update;
2. accord transitions and state synchronization;
3. Validation accept/rework and completion/archive;
4. Rules mutation;
5. Decision creation.

The same app operation must produce the same files/events/outcome regardless of interface.

### Stage 5 — isolate CLI interface ownership

- Move argument records and manual parsers to `cli/args.rs`.
- Move dispatch/help/version to `cli/mod.rs`.
- Move command adapters to `cli/commands.rs`.
- Move exact human/JSON output to `cli/output.rs`.
- Reduce `main.rs` to composition, process entry, and error/exit wiring.

Do not replace the parser or redesign output during movement.

### Stage 6 — split TUI state, input, projection, and rendering

- Extract reload/fingerprint/selection restoration.
- Introduce one small shared UI action value for keyboard/mouse convergence if behavior tests justify it.
- Extract Validation adapter behavior.
- Extract Board projection using canonical protocol hierarchy results.
- Extract Board rendering and frame-local hit geometry.
- Extract shared text and chrome only after call sites prove the boundary.
- Group feature state in separate commits after method movement is green.
- Keep Rules and Decisions feature-vertical unless concrete pressure justifies another split.

### Stage 7 — tighten, synchronize, and finish

- Synchronize final `main` changes into the integration branch.
- Remove broad imports and temporary visibility.
- Remove all temporary lint expectations.
- Confirm one hierarchy/ID/workflow/accord/review implementation.
- Confirm CLI/TUI durable mutations use app operations.
- Run all unit, real-command, concurrency, project-file, PTY, rendering, and human TUI checks.
- Update all affected docs and agent guidance to final paths and ownership.
- Review the final branch as an architecture change before merging to `main`.

## Documentation and agent-guidance scope

The Epic should explicitly review and update, where applicable:

- `AGENTS.md`;
- root `README.md`;
- `plan/spec.md` and this refactor specification;
- `protocol/README.md` and `protocol/plan/spec.md` implementation-mapping language;
- `tandem/README.md` and `tandem/plan/spec.md`;
- `tandem/plan/todo.md` and root `plan/todo.md` when milestones change;
- `extensions/README.md` and `extensions/pi-tandem/README.md` to preserve the thin-CLI-adapter rule;
- code-level module documentation for `protocol`, `project`, `app`, `cli`, and `tui`;
- agent prompts/rules that currently refer to `main.rs`, `tui.rs`, or the legacy root `Workspace` ownership model.

Documentation should continue to state:

- `protocol/` Markdown is normative;
- `tandem/src/protocol/` is the executable implementation;
- `pi-tandem` calls the CLI and does not import/reimplement protocol behavior;
- CLI and TUI are peer interfaces over shared app/protocol behavior;
- module extraction is not a protocol or user-interface redesign.

## Expected size and project shape

Current Rust source is roughly 22,774 physical lines across seven files. The refactor is not expected to reduce that dramatically.

Realistic end state:

- roughly 25–35 Rust files;
- approximately 22,000–24,000 total lines after accessors/tests are added and duplicated orchestration is consolidated;
- `main.rs` below roughly 100 lines;
- `tui/mod.rs` below roughly 500 lines;
- most modules between 200 and 1,200 lines;
- Board rendering/projection and Theme may remain approximately 1,300–2,000 lines where cohesion justifies it;
- files over roughly 2,000 lines require an ownership rationale, not an automatic split.

Line ranges are review aids, not CI rules.

## Risks and mitigations

### Long-lived branch conflict

- Risk: critical fixes or eligible non-Rust changes on `main` still require synchronization.
- Mitigation: freeze Rust implementation work on `main`, integrate unavoidable fixes immediately, synchronize at explicit checkpoints, and keep attributable move commits.

### Accidental behavior redesign

- Risk: module movement becomes an excuse to change parsing, output, protocol, or TUI behavior.
- Mitigation: real-command tests, move-only commits, existing unit/render tests, and separate accepted follow-ups for behavior changes.

### False protocol duplication

- Risk: protocol rules reappear in `project` or TUI projections.
- Mitigation: one `protocol` authority, explicit imports, dependency review, and canonical hierarchy tests.

### Excessive fragmentation

- Risk: dozens of tiny files make navigation worse.
- Mitigation: create files only around cohesive ownership; retain vertical feature modules where splitting gives no benefit.

### Visibility inflation

- Risk: moving siblings causes broad `pub(crate)` fields and APIs.
- Mitigation: move tests with code, use accessors, prefer `pub(super)`, and include a final visibility-tightening stage.

### Warning suppression becomes permanent

- Risk: temporary lint annotations hide debt forever.
- Mitigation: each annotation names a removal checkpoint, each module checkpoint removes its annotations, and final merge rejects remaining temporary suppressions.

## Completion criteria

The refactor branch is ready to merge only when:

1. the accepted architecture decision and implementation agree;
2. normative protocol docs and executable-protocol module docs are clearly distinguished;
3. protocol, project, app, CLI, and TUI ownership is visible in the source tree;
4. `main.rs` and `tui/mod.rs` are wiring roots rather than behavior warehouses;
5. there is one canonical role/relationship/ID/workflow/accord/review implementation;
6. `project::TandemProject` owns concrete `.tandem` filesystem operations and no protocol inference;
7. CLI and TUI use shared app mutations;
8. real-command tests preserve process behavior and files/events;
9. all existing and relocated unit/render/concurrency tests pass;
10. strict Clippy passes with zero diagnostics;
11. every temporary refactor lint expectation is removed;
12. visible TUI behavior receives final human validation;
13. documentation and agent guidance use final module names and boundaries;
14. the integration branch is synchronized with `main`, clean, and reviewable;
15. no new production crate, package, public Rust API, or unapproved dependency was introduced.

## Resolved review choices

1. Use `project`, not `workspace` or `persistence`, for concrete `.tandem` filesystem behavior.
2. Use `project::TandemProject` as the main discovered/opened project type.
3. Split file-format responsibility: protocol owns interpretation/validation; project owns file access and raw minimal patches.
4. Allow projects to define additional active workflow states beyond the canonical defaults.
5. Support only built-in `task` and `decision` document types for creation and first-class behavior; preserve existing custom-type data for deprecated read-only list/show/search access after upgrade.
6. Use optional fixed priority values `low`, `medium`, `high`, and `critical`.
7. Use optional fixed effort values `trivial`, `small`, `medium`, and `large`.
8. Always warn but still allow completion when review/accord acceptance is missing; structural errors still block.
9. Preserve legacy project completion settings with a deprecation warning, ignore their values, and apply canonical completion behavior.
10. Set the new protocol version to `0.2.0`.
11. Reject every project operation for `0.1.0` projects except an explicit `tandem upgrade`; do not upgrade implicitly, and defer detailed conversion mechanics to that command's implementation Task.
12. Freeze Rust CLI/TUI implementation work on `main` while the refactor branch is active.
13. Isolate existing Clippy debt with precise temporary `#[expect]` annotations and keep strict Clippy green.
14. Remove every temporary suppression before merge; separately reviewed intentional suppressions may remain.
15. Add tests that run the compiled `tandem` command in temporary projects.
16. Lock layer names, ownership, and dependency rules in the architecture decision while allowing leaf files and size ranges to evolve.

No architecture or compatibility questions remain open in this specification.
