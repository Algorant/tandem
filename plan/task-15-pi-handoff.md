# Task 15 Pi handoff

## Version and scope

- Implemented against Tandem `0.12.3`; Task-13 native evidence validation is integrated on main as `a7a60dc` (not merged or rebased into this Worker).
- Production changes are limited to `tandem/src/tui/**`.
- No Pi/session scraping, agent-launch controls, separate workflow state machine, inline milestone store, or broad layout redesign was added.

## Durable TUI workflow

The Board now has an `a` Actions picker. It uses the native protocol transition predicate to disable invalid Accord transitions and routes every mutation to the existing app layer:

- `app::accord::transition` with `AccordOptions` for claim, deliver, block, and resume.
- `app::tasks::complete` for routine completion/archive.
- `v` remains the separate exceptional human Validation picker for accept/rework.

Required prompts are transient only. Claim asks for an assignee, deliver asks for summary and required evidence, block asks for a note, and complete confirms. Native errors are shown in the footer after stale records or invalid transitions. Cancellation does not write.

The Board detail now presents native `accord.acceptance`, top-level `blockers`, child/log milestone progress, delivery summary, evidence, and changed files. Existing hierarchy-derived child context remains the source for progress; no duplicate checklist state was added.

## Task-6 reconciliation

Two lightweight approaches were compared:

1. **One contextual picker (implemented):** `a` opens the selected Task's action list, shows the current state/Accord context, disables native-invalid transitions, and opens only the required input prompt. This keeps the existing Board State/Epic layout and mouse hit-map grammar.
2. **Direct keys or CLI-only lifecycle:** several dedicated keys could call actions directly, or the TUI could remain read-only and print CLI hints. This avoids a modal but hides required inputs and makes claim/deliver/block/resume discoverability and mouse parity poor.

Recommendation: the bounded picker is the smallest durable parity interface. `task-6` remains an unclaimed research proposal and was not mutated or treated as completed research.

## Native API dependency

The TUI continues to use the existing `app::accord::transition` and `app::tasks::complete` signatures. After the Task-13 integration, the delivery prompt also calls native `protocol::accord::validate_delivery_evidence(&[String]) -> Result<(), String>` before invoking the app layer. The prompt treats evidence as one opaque item so commas in prose are preserved; empty, whitespace-only, and punctuation-only comma input stays in the prompt. Native app validation remains final and rejects before any record/event write with `accord deliver requires at least one non-empty --evidence <text>`.

## Validation

- Before adding the Task-13 API call, old-base `cargo test --manifest-path tandem/Cargo.toml tui::tests` passed **111 tests**. Those results are TUI regression evidence only and are not proof of the integrated native fix; rerun after refreshing this source onto main `a7a60dc`.
- Added actual keyboard and mouse picker/modal paths, missing assignee/block note/evidence assertions, comma-preserving evidence, cancellation, stale-record native error retention, role-correct Task/Subtask rendering, archived completed/canceled/failed outcomes, long detail scrolling, and final log evidence assertions.
- `cargo build --manifest-path tandem/Cargo.toml --release`: passed.
- `cargo fmt --manifest-path tandem/Cargo.toml -- --check`: passed after formatting changed TUI files.
- Release ANSI inspection: launched the release TUI in Herdr pane `w4P:p2` and read `--format ansi`. Board rendered title, state tabs/counts, selected task row, and `a Actions · e Edit · f Filter · v Validate · b Epic Board · ? Help`. The action picker also rendered in the narrow preview pane. Temporal flicker/resize latency and taste criteria remain unverified.

## Reproducible preview

The disposable fixture was created and then deliberately enriched with the native CLI. Task 1 retains the owner probe's delivered empty-evidence state, now also has blocker `task-2` and milestone child `task-1-1`; archived `task-2` contains valid final evidence. This is an intentional mixed-state preview, not an untouched initial workspace.

The disposable fixture is at:

`/tmp/tandem-task-15-preview`

Create/run commands:

```sh
rm -rf /tmp/tandem-task-15-preview
mkdir -p /tmp/tandem-task-15-preview
cd /tmp/tandem-task-15-preview
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem init --title 'Task 15 TUI workflow preview'
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem add task 'Preview durable workflow actions' --acceptance 'Claim, deliver, block, resume, and complete from the TUI' --acceptance 'Show acceptance, blockers, milestone progress, and evidence'
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem add task 'Preview blocking dependency' --acceptance 'Dependency can be archived before parent completion'
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem add task 'Preview milestone child' --acceptance 'Milestone status appears under its parent Task' --parent task-1
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem update task-1 --blocker task-2
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem accord claim task-2 --assignee preview
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem accord deliver task-2 --summary 'Dependency evidence recorded' --evidence 'native CLI deliver and show'
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem complete task-2
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem tui
```

## Consumer/version handoff

The consumer is the native Rust TUI in the same `tandem` `0.12.3` binary. No extension or Pi adapter changes are required. Task-13 should reconcile native deliver evidence validation before claiming complete parity.
