# Research: modularizing the large Rust CLI and TUI modules

Date: 2026-07-22

Task: `task-145`

Audited baseline: `98a96e60ce05fdf724cf2d54cbb29ca3c0359b28` (`tandem` 0.6.0)

## Recommendation

Use a **binary-only, leaf-first staged extraction**. Keep the existing `tandem/` Cargo package and its single binary crate, move cohesive code into ordinary Rust modules under `src/`, and reduce `main.rs` and `tui.rs` to wiring. Do not add a root workspace, another package, a library target, a component framework, or broad trait layers during this refactor.

The preferred dependency direction is:

```text
main.rs
  -> cli (process arguments, dispatch, output)
      -> app (typed queries/use cases and outcomes)
      -> tui (only for the `tui` command)

cli + tui
  -> app + model

tui Board/review projection
  -> hierarchy (read-only canonical role/relationship queries)

app
  -> hierarchy (the only role/relationship/ID authority)
  -> workspace (discovery, reads, raw patches, locking, writes, events)
  -> model

hierarchy + workspace
  -> model + error
```

Start with exact code moves at low-coupling leaves. Preserve function bodies and move their tests in the same review. Make algorithm, type-shape, or API improvements only in a later review after the move is green. This keeps each diff attributable and avoids turning modularization into a rewrite.

The best first extraction is the current editor seam at `tui.rs:3652-3797`: move `EditorTarget`, `EditorCommand`, editor selection/environment parsing, command execution, and their focused tests to `tui/editor.rs`. Leave `TuiApp::open_selected_item_in_editor` in place initially. This is about 146 production lines, has narrow dependencies, and already has parser/target/process-smoke coverage.

## Scope and measurement method

This report inventories the seven tracked Rust source files at current HEAD. Physical line counts came from `wc -l`; test counts came from `#[test]`; history counts came from `git log`; churn is historical additions plus deletions from `git log --numstat`. Current-range hotspot counts use `git log -L` and therefore indicate how many distinct commits participate in that evolving line history, not how many times each line changed.

“Body before final test module” below means lines before the final `mod tests {` declaration. It is not semantic production LOC: the preceding `#[cfg(test)]` attribute and several test-only wrappers occur outside those final modules.

## Quantitative baseline

### Source size, tests, and whole-file churn

| File | Physical lines | Body before final `mod tests` | Final test module | `#[test]` count | Path-touching commits | Additions + deletions |
|---|---:|---:|---:|---:|---:|---:|
| `src/main.rs` | 6,724 | 5,372 | 1,352 | 37 | 17 | 7,525 + 801 = 8,326 |
| `src/tui.rs` | 10,172 | 7,328 | 2,844 | 79 | 40 | 11,710 + 1,552 = 13,262 |
| `src/tui/decisions.rs` | 1,054 | 936 | 118 | 5 | 7 | 1,284 + 230 = 1,514 |
| `src/tui/logs.rs` | 823 | 595 | 228 | 8 | 7 | 1,085 + 262 = 1,347 |
| `src/tui/review.rs` | 1,002 | 866 | 136 | 3 | 4 | 1,026 + 24 = 1,050 |
| `src/tui/rules.rs` | 1,047 | 1,019 | 28 | 2 | 6 | 1,186 + 141 = 1,327 |
| `src/tui/theme.rs` | 1,952 | 1,423 | 529 | 20 | 14 | 2,256 + 304 = 2,560 |
| **Total** | **22,774** | **17,539** | **5,235** | **154** | — | **29,386 changed lines** |

Key concentration signals:

- `main.rs` plus `tui.rs` contain 16,896 lines, **74.2%** of all tracked Rust source in the crate.
- Those two files contain 116 of 154 tests, **75.3%** of the test cases.
- They account for 21,588 historical changed lines, **73.5%** of measured source churn.
- There is no `src/lib.rs` and no `tandem/tests/` target. All 154 tests are unit tests in the binary crate.
- Current test baseline is `154 passed; 0 failed; 0 ignored`.
- Strict Clippy is **not green at current HEAD** with Rust/Clippy 1.96: `cargo clippy --all-targets -- -D warnings` exits 101 with 27 binary-target diagnostics and 28 test-target diagnostics in pre-existing Rust code. The reported categories include `too_many_arguments`, `unnecessary_map_or`, `manual_checked_ops`, and several smaller style lints. A separate hygiene prerequisite should establish a green lint baseline before strict Clippy becomes a stage gate.

### Expensive and high-churn seams

| Current seam | Size | Distinct commits in current-range line history | Why it is costly |
|---|---:|---:|---|
| TUI Board projection/filter/row/preview helpers, `4571-6728` | 2,158 lines | 25 | Hierarchy, filtering, expansion, responsive rows, previews, and theme signals move together. |
| `TuiApp::handle_key`, `686-799` | 114 lines | 17 | Global shortcuts, modal ownership, view routing, and mutations are interleaved. |
| `TuiApp::reload`, `473-594` | 122 lines | 10 | Locking, tolerant reads, hierarchy, themes, logs/events, warnings, and selection restoration converge. |
| `TuiApp::handle_mouse`, `1608-1745` | 138 lines | 10 | Hit interpretation, focus, scrolling, and direct state changes converge. |
| CLI command adapter/orchestration region, `1323-2302` | 980 lines | 14 | Workspace discovery, use cases, printing, and command policy are mixed. |
| CLI task mutation/validation region, `2787-3642` | 856 lines | 12 | Task, accord, hierarchy, persistence, and display concerns cross. |
| CLI JSON output region, `4439-4876` | 438 lines | 11 | Hand-built stable envelopes depend on many model accessors and hierarchy summaries. |
| Canonical `HierarchyIndex` block, `151-388` | 238 lines | 3 | Its history is newer, but its semantic blast radius is high: CLI and TUI correctness depend on one authority. |

The 3,225-line `impl TuiApp` at `427-3651` contains **138 methods**. `TuiApp` itself has **38 direct fields**. Long methods are only part of the problem: many short navigation, selection, drawing, and prompt methods all mutate the same aggregate.

A large simultaneous move of either root file would collide with active work even if behavior were unchanged. Four non-merge commits already changed both roots together, including hierarchy alignment, concurrent allocation, and accord state synchronization.

## Responsibility and dependency inventory

### `main.rs`

| Current lines | Size | Responsibilities and major symbol clusters |
|---|---:|---|
| `1-617` | 617 | imports/constants; `CliError`; `Workspace`, `Document`, `DocumentLocation`; canonical `TaskRole`, `ParentRelationship`, `HierarchyIndex`; `HierarchyLock`; CLI option/outcome records |
| `618-1322` | 705 | process entry and exit; command dispatch; help/version; manual argument parsers and common parser helpers |
| `1323-2312` | 990 | command adapters and orchestration for init/tasks/logs/accord/rules/decisions/TUI |
| `2313-2733` | 421 | workspace discovery; document reads/lookups; hierarchy children; frontmatter/YAML parsing and flattening |
| `2734-3642` | 909 | filters; move/update mutation; metadata validation; state/accord/review interpretation; canonical mutation validation; accord transitions |
| `3643-3981` | 339 | sorting; human tables/details; search projection/snippets; log/decision presentation |
| `3982-4438` | 457 | rules model/parsing; completion/accord/rules minimal-diff renderers and patchers; rules printing |
| `4439-4876` | 438 | JSON envelopes, summaries, metadata fields, and escaping |
| `4877-5371` | 495 | state validation; sequential creation/allocation; generic frontmatter patching; snapshots and atomic writes; event append; YAML/list/time/path/string helpers |
| `5372-6724` | 1,353 | final test attribute/module region with 37 tests |

The file has at least six architectural responsibilities:

1. process/CLI syntax;
2. protocol model and metadata interpretation;
3. canonical hierarchy and validation;
4. filesystem persistence and concurrency;
5. application use cases;
6. human/JSON presentation.

The important shared state is concrete rather than global:

- `Workspace` is four paths (`board`, `logs`, config, events).
- `Document` combines path/location, flattened frontmatter fields, and raw body.
- `HierarchyIndex` owns the board-plus-logs document graph used for role/relationship validation and allocation.
- `HierarchyLock` serializes graph snapshots and mutation through the config file.
- `TEMP_FILE_COUNTER` is the only mutable static and is limited to temporary-file naming.

This is a good reason to extract concrete modules, not a reason to introduce dependency injection.

### `tui.rs`

| Current lines | Size | Responsibilities and major symbol clusters |
|---|---:|---|
| `1-426` | 426 | terminal lifecycle; views/focus/actions/hits; quick-add and Validation prompts; Board filters/arrangement; reload snapshots; 38-field `TuiApp` |
| `427-3651` | 3,225 | load/reload/event loop; keyboard/mouse; mutations; selection/navigation; top-level, Board, Logs, header/footer/help rendering; hit registration |
| `3652-3797` | 146 | editor target, environment command parsing, and process execution |
| `3798-4183` | 386 | workspace state parsing; tolerant reads; hierarchy/load diagnostics; fingerprints; quick-add selection/status helpers |
| `4184-4570` | 387 | quick-add and Validation accept/rework/apply mutations; feedback; Board subview metadata |
| `4571-6728` | 2,158 | Board hierarchy projections; entry/view models; recursive filtering; relationship context; rows/chips; inline previews; legacy inline checklist display |
| `6729-7327` | 599 | Board details/accord hints; Markdown-ish rendering and wrapping; shared layout/status helpers |
| `7328-10172` | 2,845 | final test attribute/module region with 79 tests |

The 38 `TuiApp` fields mix distinct lifetimes and ownership domains:

1. durable workspace data and canonical hierarchy snapshot;
2. loaded config, theme, rules, logs, events, and warnings;
3. per-view selection, filter, expansion, focus, and scroll state;
4. modal/input state;
5. transient status and reload timing;
6. frame-derived mouse hit regions.

Grouping those fields into feature state is justified eventually. It should be a separate behavior-preserving change after code movement, not part of the initial extraction.

### Existing `tui/` module tree

| Module | Ownership today | Important coupling |
|---|---|---|
| `theme.rs` | built-in palettes, user/workspace theme discovery and precedence, badge/display config, style methods | depends only on `Workspace`/path display from the root; cohesive and well tested |
| `logs.rs` | tolerant log/event loading, filtering, list rows, detail projection | consumes root document/completion accessors and canonical `HierarchyIndex` context |
| `review.rs` | legacy review queue projection and rendering | declared with `#[allow(dead_code)]`; no `TuiView::Review` remains, but its hierarchy-aware tests still compile |
| `rules.rs` | Rules state, prompts, input, mutations, and rendering via an `impl TuiApp` block | directly calls root snapshots, raw patching, atomic writes, and events |
| `decisions.rs` | Decisions state, prompts, input, mutations, and rendering via an `impl TuiApp` block | directly calls root sequential creation, timestamps, and events |

The existing Rules and Decisions modules show a useful **feature-vertical** pattern. They should remain cohesive for now; splitting each mechanically into state/input/update/render files would add navigation and visibility costs without attacking the Board/root hotspot.

### Current dependency and coupling hotspots

The current module tree has no Rust crate cycle, but it has a conceptual adapter/core cycle:

```text
main/root dispatch -> tui::run_tui

tui.rs -- `use super::*` --> nearly every category of root internals

tui/{decisions,logs,review,rules,theme}
  -> explicit imports from both tui.rs and crate root
```

Specific hotspots are:

- **Implicit root API:** production `tui.rs` uses `use super::*`. Root-private types, path fields, patchers, validators, formatters, and mutation helpers form an untracked internal API.
- **Direct model internals:** TUI code relies on root-private `Document`/`Workspace` fields because it is a descendant module. Moving those types to sibling modules will require narrow accessors or carefully scoped visibility.
- **Shared-versus-duplicated mutations:** TUI state movement already calls `move_task_to_state`, but quick-add, Validation completion, Rules mutation, and Decision creation each orchestrate persistence themselves. The CLI has parallel orchestration. This is the strongest case for typed application use cases.
- **Reload fan-in:** one method constructs all durable and derived TUI state, handles tolerant error policy, and restores every feature selection.
- **Render/input geometry:** rendering registers frame-local hit regions later consumed by mouse input. Hit generation must remain next to layout; only action application should move away from rendering.
- **Feature modules extending `TuiApp`:** this works while `TuiApp` lives in the parent `tui` module. Moving the struct to a sibling `app.rs` immediately would force broad `pub(super)` fields or accessors. Keep the aggregate in `tui.rs` until feature state boundaries are ready.
- **Canonical hierarchy blast radius:** Board projection may arrange resolved documents, but role, relationship, and valid-ID inference must remain in `HierarchyIndex`; no extraction may copy that logic into TUI view models.
- **Colocated tests:** test modules intentionally use parent-private details. Moving implementation without its tests will either break privacy or encourage unnecessary public APIs.

## Proposed target module map

This is a target direction, not a request to create every path at once. A file should appear only when code moves into it.

```text
tandem/src/
├── main.rs                   # module declarations + process exit/error wiring only
├── error.rs                  # existing CliError semantics and exit category accessors
├── model.rs                  # Document, DocumentLocation, protocol metadata accessors/values
├── hierarchy.rs              # sole TaskRole/relationship/ID/graph authority
├── workspace/
│   ├── mod.rs                # Workspace paths, discovery, strict/tolerant document reads
│   ├── frontmatter.rs        # YAML flattening and raw minimal-diff patch primitives
│   ├── write.rs              # snapshots, conflict checks, atomic/new writes, hierarchy lock
│   └── events.rs             # append-only audit event encoding/appends
├── app/
│   ├── mod.rs                # shared snapshot/query surface and typed outcomes
│   ├── tasks.rs              # add/move/update/complete use cases
│   ├── accord.rs             # accord and Validation transitions
│   ├── decisions.rs          # decision creation/diagnostics
│   └── rules.rs              # rule mutations
├── cli/
│   ├── mod.rs                # run-from-args, dispatch, help/version
│   ├── args.rs               # option structs and manual parsers
│   ├── commands.rs           # thin adapters from parsed options to app use cases
│   └── output.rs             # exact human and JSON rendering
├── tui.rs                    # TuiApp aggregate, module wiring, startup/event-loop shell
└── tui/
    ├── terminal.rs           # raw/alternate-screen/mouse enter, suspend, resume, Drop cleanup
    ├── editor.rs             # editor target/env parsing and process execution
    ├── reload.rs             # load/reload/fingerprint/selection restoration
    ├── input.rs              # crossterm key/mouse translation and action dispatch
    ├── validation.rs         # Validation prompt state/actions and UI adapter logic
    ├── chrome.rs             # top tabs/header/footer/help and their frame hit geometry
    ├── text.rs               # genuinely shared Markdown-ish/detail/wrapping helpers
    ├── board/
    │   ├── mod.rs            # BoardState and feature-level dispatch
    │   ├── projection.rs     # canonical-index-backed entries, filtering, relationship context
    │   └── render.rs         # rows/detail/preview/layout and frame-local hit registration
    ├── decisions.rs          # keep current feature-vertical state/input/render ownership
    ├── logs.rs               # keep current feature-vertical loading/projection/render ownership
    ├── review.rs             # retain until a separate removal/product decision
    ├── rules.rs              # keep current feature-vertical state/input/render ownership
    └── theme.rs              # keep current theme/config/style ownership
```

Keeping `tui.rs` as the module root is idiomatic and avoids a path-only rename to `tui/mod.rs`. It can still become thin while retaining the parent-private `TuiApp` aggregate that child feature modules can use. Start Board extraction one directional seam at a time; do not move all 2,700-plus Board/detail/text lines in one commit merely to match the map.

### Ownership and API rules

- `error` keeps existing error text and numeric category behavior. Do not rename or redesign errors during extraction.
- `model` defines data and metadata accessors; it does not discover files, print, or render. Prefer narrow `Document` accessors over making all fields `pub(crate)`.
- `hierarchy` is the only authority for Epic/Task/Subtask roles, relationships, structural validity, and role-specific ID forms. It should eventually construct from documents, not read the filesystem itself.
- `workspace` owns file representation and side effects, including raw-source preservation, locking, conflict checks, and atomicity. It does not infer hierarchy roles.
- `app` acquires/loads a coherent workspace snapshot, invokes hierarchy checks, and composes use cases. It returns typed outcomes/warnings and never prints or renders.
- `cli` owns strings at the process boundary: syntax, usage errors, stdout/stderr, table labels, and JSON envelopes.
- `tui` owns terminal events, transient UI state, projection, and rendering. Rendering may update Ratatui widget state and frame hit regions, but must not write project files.
- Default to private items. Use `pub(super)` for child-to-parent feature seams and `pub(crate)` only for genuine cross-adapter/application APIs. Do not use unqualified `pub` without an intentionally approved library API.
- Replace the production wildcard import with explicit imports as each seam settles. Do not perform a repository-wide import cleanup in the first move commit.

## Data-flow boundaries

### CLI

```text
Vec<String>
  -> cli::args typed command options
  -> cli::commands adapter
  -> app query/use case
  -> typed outcome + warnings
  -> cli::output exact human or JSON output
```

Manual parsing and hand-built JSON are not the reason the file is too large. Replacing them with `clap` or `serde` during extraction would change dependencies and risk help, error, omission, order, and escaping behavior. Move them unchanged first; evaluate replacements only as separate product decisions.

### TUI

```text
crossterm Event
  -> input translation (UiAction)
  -> TuiApp/feature-state update
  -> app use case for durable mutation
  -> reload coherent durable state
  -> feature projection
  -> Ratatui render + frame-local HitRegion map
```

A `UiAction` enum is justified because keyboard and mouse already converge imperfectly through `KeyAction` and `HitAction`. It can remove duplicated action application without requiring a full Elm/Redux rewrite. Ratatui stateful widgets and Tandem's hit map make pragmatic mutable rendering appropriate.

### Shared use cases

Existing concrete functions and outcomes are the right starting seams: task add/move/update, accord application, completion patching, sequential creation, rules updates, and decision creation. Move and compose those before considering service objects.

Useful new types are limited to demonstrated ownership gaps:

- adapter-independent use-case inputs/outcomes;
- `UiAction` for keyboard/mouse convergence;
- feature state such as `BoardState` or reload/modal state once field moves are separately covered.

A `WorkspaceSnapshot` may become useful if board documents, logs, and `HierarchyIndex` are repeatedly passed as one coherent value. Do not add it until two use cases demonstrate that invariant.

The existing generic sequential-document creator is justified: its closure builds content inside an atomic retry/allocation operation. It does not imply a need for more generic storage abstractions.

## Plausible plans of attack

### Plan A — binary-only, leaf-first extraction (preferred)

Leave `main.rs` as the binary crate root and add ordinary child modules. Perform exact moves from low-coupling leaves toward shared core, then reduce visibility and reshape state in later reviews.

**Advantages**

- Preserves the one-package/one-binary-crate v0 constraint and all current dependencies.
- Keeps `pub(crate)` internal to one crate and allows colocated unit tests to access private implementation.
- A module move does not add a Cargo target or new test harness.
- Small move-only reviews can be rebased around feature work and validated with all 154 tests.
- The future `lib.rs` option remains open after internal boundaries have evidence.

**Costs and risks**

- Rust still compiles one crate; more files do not create independent compilation units or guarantee faster builds.
- The first move of a widely shared root type can trigger many privacy/import edits even with identical behavior.
- Temporary `pub(crate)` seams may be wider than the final design and need an explicit tightening stage.
- Unit tests remain internal; external contract coverage must launch the binary.

**Merge strategy**

- Rebase before each move, especially around hierarchy and Board work.
- Move one cohesive seam plus its tests; do not rename/reformat/rewrite it simultaneously.
- Land behavior/API cleanup only after the move commit is accepted.

### Plan B — add `lib.rs` first, then make the binary call the library

A Cargo package can contain both `src/main.rs` and `src/lib.rs`, but that is **two crates**: one binary and one library.

**Advantages**

- Enforces the process/library dependency boundary with the compiler.
- Makes a tiny binary immediate.
- Allows integration tests and a future embedded consumer to import application APIs.

**Costs and risks**

- It conflicts with the current “single binary crate” direction unless that decision is explicitly revised.
- `pub(crate)` in the library is not visible to the binary crate, creating pressure to declare a public API before ownership has stabilized.
- A safe conversion must move code wholesale into the library; declaring modules from both roots would compile duplicate copies/types.
- Cargo gains another target/test harness, and the initial import/privacy diff spans both hot roots.
- It has the highest merge-conflict and review risk because structural and API decisions arrive together.

**Assessment:** idiomatic when Tandem has a real second consumer or wants a supported Rust API, but not the safest v0 modularization step.

### Plan C — vertical feature extraction first

Move a command/view family end to end—for example Rules, Decisions, then Logs—while leaving shared model/persistence code in `main.rs`.

**Advantages**

- Each review has a recognizable product feature boundary.
- Existing Rules and Decisions modules already demonstrate the pattern.
- Work can avoid the hottest Board region for longer.

**Costs and risks**

- Shared root internals remain an implicit API, so feature files continue to import persistence and hierarchy details directly.
- It can cement parallel CLI/TUI mutation orchestration instead of creating shared use cases.
- Board, reload, and root output remain unsolved.
- Later core extraction touches every feature again, increasing total migration work.

**Assessment:** a useful fallback if ongoing feature churn prevents a short core-extraction window, but inferior as the primary dependency strategy. Combine selective feature moves with Plan A rather than treating vertical slices as the final architecture.

## Preferred staged approach

Each numbered item is an independently reviewable follow-up. Do not mix feature behavior with these moves.

### Stage 0 — freeze observable contracts

1. Record the current `cargo test` baseline.
2. Resolve or explicitly baseline the current Rust/Clippy 1.96 `-D warnings` diagnostics in a separate hygiene change; do not mix those source edits into a move commit.
3. Add a small black-box `tests/cli_contract.rs` only if existing tests or other landed work do not already provide equivalent coverage. Launch `CARGO_BIN_EXE_tandem` in temporary workspaces created by test code; do not add fixture directories or dependencies.
4. Cover exact stdout, stderr, and exit status for help/version; unknown command/flag; missing workspace/document; representative add/move/complete; and at least one human and JSON read.
5. Keep focused `TestBackend` assertions for important TUI rows/layout/hits; avoid whole-screen snapshots for every view.

### Stage 1 — prove the extraction pattern on TUI leaves

1. **Initial extraction:** move `tui.rs:3652-3797` and the editor target/parser/process tests into `tui/editor.rs`. Keep function bodies and visible messages unchanged.
2. Separately move `TerminalSession`, `restore_terminal`, and enter/suspend/resume/Drop behavior into `tui/terminal.rs`.
3. Run focused editor tests plus the full suite after the first move. After terminal lifecycle moves, repeat the documented PTY smoke because alternate-screen/raw-mode cleanup cannot be fully proven by ordinary unit tests.

These seams avoid Board projection and hierarchy ownership while establishing explicit import and `pub(super)` conventions.

### Stage 2 — isolate CLI syntax and presentation

1. Move each option record with its parser to `cli/args.rs`; preserve manual parsing and exact usage errors.
2. Move dispatch/help/version to `cli/mod.rs`; reduce `main.rs` to module declarations and process error/exit handling.
3. Move command adapters to `cli/commands.rs` and human/JSON formatting to `cli/output.rs` without changing output strings, ordering, omission rules, or warning placement.
4. If a temporary `pub(crate)` parser/output seam is needed, record and tighten it before the stage closes.

### Stage 3 — establish model, workspace, and hierarchy ownership

Use separate reviews in this order:

1. Move `CliError` and `Document`/`DocumentLocation`, adding only the narrow accessors required by sibling modules.
2. Move frontmatter parsing/patching and their preservation tests.
3. Move snapshots, atomic writes, sequential new-file creation, locking, and events without changing ordering or failure behavior.
4. Move `TaskRole`, `ParentRelationship`, `HierarchyIndex`, role-specific ID parsing/allocation queries, and hierarchy tests as one canonical semantic unit.
5. In a later cleanup review, have `app` load documents and call `HierarchyIndex::from_documents`; remove filesystem discovery from the hierarchy module rather than introducing a workspace/hierarchy dependency cycle.

This is the highest semantic-risk stage. Rebase over current hierarchy work and do not change role/relationship algorithms in a move commit.

### Stage 4 — unify typed application use cases

Move one mutation family per review and switch both adapters before proceeding:

1. add/move/update;
2. accord transitions and state synchronization;
3. Validation accept/rework/apply plus completion/archive;
4. Rules mutations;
5. Decision creation.

CLI and TUI may still perform adapter-specific input validation and messaging, but filesystem mutation, hierarchy checks, locking, event append, and typed outcomes should be shared. Compare resulting Markdown and JSONL bytes in focused tests where practical.

### Stage 5 — split TUI state/input/update/render along observed seams

1. Move reload/fingerprint/selection restoration to `tui/reload.rs`.
2. Introduce one small `UiAction` and make keyboard/mouse routes share action application; keep frame hit generation next to render geometry.
3. Move canonical-index-backed Board entry projection/filtering to `tui/board/projection.rs` with its hierarchy tests.
4. Move Board row/detail/preview/layout and hit registration to `tui/board/render.rs` with `TestBackend` coverage.
5. Move Validation prompt/update adapter logic to `tui/validation.rs`.
6. Move shared Markdown/detail helpers to `tui/text.rs`, and top-level tabs/footer/help to `tui/chrome.rs`, only after call sites establish the boundary.
7. Group fields into feature state (`BoardState`, log/modal/reload state) in separate changes after the method moves compile and pass.

Keep `TuiApp` in parent `tui.rs` during this campaign so child modules retain private descendant access. Do not split Rules or Decisions merely for symmetry.

### Stage 6 — tighten boundaries and reconsider a library only with evidence

- Replace remaining broad imports with explicit imports.
- Reduce temporary `pub(crate)` to `pub(super)` or private.
- Audit that `workspace` does not infer hierarchy and render modules do not write files.
- Remove dead Review code only in a separate product/cleanup decision.
- Reconsider `lib.rs` only when a real second Rust consumer, embedding requirement, or supported library API justifies revisiting the one-binary-crate constraint.

## Invariants and regression coverage

### Coverage by stage

| Stage | Required focused evidence before and after |
|---|---|
| Editor/terminal leaves | editor quoting/target/process tests; terminal PTY enter/exit/editor suspend-resume smoke; full suite |
| CLI syntax/output | black-box stdout/stderr/exit contracts; parser unit tests; representative human and JSON reads; full suite |
| Model/workspace/hierarchy | YAML/frontmatter preservation; atomic/conflict/event tests; concurrent global/Subtask allocation; all hierarchy role/error cases; full suite |
| Shared app use cases | CLI/TUI parity for files, events, warnings, and outcomes; no-op/cancel leaves files unchanged; full suite |
| TUI split | key/modal ownership; keyboard/mouse equivalence; hit/scroll geometry; reload selection; Board hierarchy/filter/render buffers; PTY smoke; full suite |

Every production stage should run:

```sh
cargo fmt --manifest-path tandem/Cargo.toml -- --check
cargo test --manifest-path tandem/Cargo.toml
cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings
```

Strict Clippy should become a required green gate after Stage 0 establishes a clean baseline. Until then, a move must introduce no additional diagnostics and must report the known baseline explicitly; silently accepting an ever-changing warning count is not sufficient. A move-only diff should not reduce the test count unless the same coverage is demonstrably relocated or consolidated.

### CLI invariants

- Canonical command names, long flags, positional handling, help/version text, and usage diagnostics.
- Exit categories: `0` success, `1` runtime/data/write failure, `2` usage failure.
- Human table headers, row ordering, labels, whitespace, empty/no-match text, warning ordering, and stderr/stdout choice.
- JSON envelope, field names/order, omission rules, escaping, role/relationship values, warnings, and child summaries.
- Preferred `validation` writes and legacy `review` read/filter compatibility.
- Workflow state, accord state, and review metadata remain distinct.
- Unknown frontmatter fields and raw body text survive minimal-diff mutations.
- Snapshot conflict checks, atomic/new-file writes, hierarchy-lock lifetime, and event append semantics remain unchanged.
- Global Task/Epic allocation and parent-derived Subtask allocation scan both Board and Logs without reuse, including concurrent creation.
- Canonical hierarchy failures remain fail-closed for reads/mutations where currently required.
- Completion archives to Logs with the same warning policy, metadata, path, and event behavior.

Keep parser, hierarchy, patching, JSON helper, and mutation unit tests colocated with their modules. Add black-box tests only for observable CLI contracts; do not move all unit tests to `tests/` or expose internals just for tests.

### TUI invariants

- Raw mode, alternate screen, mouse capture, cursor, Drop cleanup, and editor suspend/resume behavior.
- Fixed keybindings, view/focus semantics, modal prompt ownership, quit behavior, and contextual help/footer hints.
- Keyboard and mouse action equivalence for tabs, rows, expansion, filtering, scrolling, and footer actions.
- Frame hit regions remain aligned with rendered/scroll-clipped geometry.
- State and Epic Board ordering, canonical hierarchy labels, recursive expansion, filter ancestor context, and selection preservation across reload.
- Invalid hierarchy snapshots display persistent actionable diagnostics and disable graph-sensitive mutations rather than flattening documents.
- Quick-add/move and Validation accept/rework/apply semantics; cancellation/no-op paths leave files unchanged.
- External-change fingerprints, tolerant parse/load warnings, and non-panicking reload behavior.
- Important row chips/metadata at narrow widths, inline preview caps, Markdown-ish basics, and detail sections.
- Theme discovery/fallback and user/workspace precedence; Logs/Rules/Decisions behavior and selection restoration.

Focused `TestBackend` buffer assertions are appropriate. Whole-screen snapshots would be brittle. Visual polish still needs human terminal review, but mechanical moves should preserve existing buffers and should not create new visual design decisions.

## Code that should deliberately remain together

- Canonical role resolution, parent relationship derivation, structural graph validation, and role-specific ID parsing/allocation queries in `hierarchy`.
- Filesystem snapshot/conflict/atomic-write behavior and lock lifecycle in `workspace::write`; `app` composes it with hierarchy queries.
- Raw frontmatter patch primitives with their byte/body/unknown-field preservation tests. Do not replace them with full typed reserialization during modularization.
- Each command parser beside its option record.
- Terminal enter/restore/suspend/resume/Drop behavior as one safety-critical unit.
- Board recursive projection helpers that jointly establish visible ancestors/descendants. Separate rendering only at the existing entry/view-model boundary.
- Render layout and frame-local hit registration; geometry is their shared invariant.
- Rules prompt/state/render code and Decisions prompt/state/render code for now; only their durable mutations move to `app`.
- Theme discovery, precedence, palette, badge, and display configuration.
- The compiled `review.rs` path until a separate decision removes or revives it.

## Abstractions not justified yet

- No `Repository`, `Storage`, `Renderer`, `Command`, `Component`, filesystem, or clock trait. There is one filesystem and one Ratatui runtime, and real temporary-workspace tests already exercise them.
- No generic component registry, plugin framework, event bus, dependency-injection container, or async runtime.
- No full immutable Elm/Redux conversion. A small action enum is enough.
- No newtype for every ID, status, or field string. Existing `TaskRole`, `ParentRelationship`, command inputs, and outcomes already prevent important mistakes. Add a stronger ID type only when it prevents a demonstrated cross-module invalid transition.
- No `clap`, `serde`, snapshot framework, or new test dependency as part of code movement.
- No separate core/TUI crates, root Cargo workspace, extra package, or `lib.rs` without a separate accepted decision.
- No command/output redesign, keymap redesign, protocol representation rewrite, or removal of legacy Review paths in the extraction campaign.

Ratatui documents both component traits and Elm-style Model/Update/View as optional application patterns. Tandem should take the useful data-flow ideas without adopting a framework-shaped architecture that its four fixed views do not need.

## Measurable completion criteria

The eventual refactor is complete when all of the following are true:

1. `main.rs` contains module declarations and process entry/error/exit wiring only (target: under 100 lines).
2. `tui.rs` contains the `TuiApp` aggregate plus startup/high-level event-loop wiring only (target: under 500 lines).
3. No production module exceeds roughly 2,000 physical lines without a written ownership rationale; no TUI feature file exceeds roughly 1,500 lines without a demonstrated reason. These are review triggers, not CI style laws.
4. Production code contains no `use super::*`; cross-module imports are explicit.
5. No unqualified `pub` or broad public fields exist solely to bypass module privacy. Cross-crate public API remains zero unless a library target is separately approved.
6. `Document` and `Workspace` internals are reached through narrow accessors/operations rather than broadly exposed fields.
7. Search/code review finds one canonical Task role/relationship/ID implementation and no TUI or CLI reimplementation.
8. CLI and TUI call shared application mutations for task movement/add/update, accord/Validation, completion, Rules, and Decisions; adapters only own input and presentation differences.
9. TUI durable data, Board state, view state, modal state, reload state, and frame hits have explicit ownership rather than 38 unrelated root fields.
10. The 154-test baseline is preserved or strengthened, black-box CLI contracts cover output/exit behavior, and focused TUI tests cover key/mouse/render/reload behavior.
11. `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` pass at every landed stage.
12. No dependency, Cargo target, protocol shape, command output, event shape, keybinding, or theme behavior changes in a move-only stage.
13. Git history shows independently reviewable move/ownership commits rather than one repository-wide rewrite.

## External references

- Rust Book, “Packages and Crates”: modules may live in multiple files within one crate; `src/main.rs` plus `src/lib.rs` creates two crates in one package. <https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html>
- Cargo Book, “Package Layout”: conventional binary, library, and integration-test target locations. <https://doc.rust-lang.org/cargo/guide/project-layout.html>
- Rust Reference, “Visibility and privacy”: private descendants, `pub(crate)`, and `pub(super)` semantics support narrow internal APIs and colocated tests. <https://doc.rust-lang.org/reference/visibility-and-privacy.html>
- Ratatui, “Component Architecture”: feature-local state/event/update/render is one optional pattern, not a required trait framework. <https://ratatui.rs/concepts/application-patterns/component-architecture/>
- Ratatui, “The Elm Architecture”: Model/Update/View and event-to-message separation are useful, while stateful widgets justify pragmatic mutable rendering. <https://ratatui.rs/concepts/application-patterns/the-elm-architecture/>

## Research-checkout validation

Commands run against the audited HEAD before this document was committed:

```sh
git rev-parse HEAD
wc -l tandem/src/main.rs tandem/src/tui.rs tandem/src/tui/*.rs
rg -n '^\s*#\[test\]' tandem/src | wc -l
git log --format='COMMIT %H' --numstat -- tandem/src/main.rs tandem/src/tui.rs tandem/src/tui/
git log -L <current-start>,<current-end>:<file> --format='COMMIT %H' --no-patch
cargo fmt --manifest-path tandem/Cargo.toml -- --check
cargo test --manifest-path tandem/Cargo.toml
cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings
```

Results: HEAD matched `98a96e60ce05fdf724cf2d54cbb29ca3c0359b28`; physical/source/test/history measurements matched the tables above; formatting passed; all 154 tests passed with no failures or ignored tests. Strict Clippy exited 101 on the pre-existing diagnostics summarized in the baseline section; this research task made no Rust changes.
