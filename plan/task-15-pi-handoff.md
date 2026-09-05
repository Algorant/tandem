# Task 15 Pi handoff

## Version and scope

- Implemented against Tandem `0.12.3` (`735c4cf` published baseline; parent baseline `3d179b0`).
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

## Native API request / risk

The current native signatures consumed are sufficient for durable writes. One version-matched mismatch remains for the orchestrator to reconcile with Task-13: protocol README 0.3.0 says deliver requires at least one evidence item, while the current native `app::accord::transition` input validation only requires `summary`. The TUI prompt requires evidence and passes it as `AccordOptions.evidence`, but the native app should enforce this for CLI/TUI parity. Exact requested native adjustment: `validate_accord_inputs("deliver", options)` should reject empty `options.evidence` with a usage error; no TUI workaround should replace that app-layer rule.

## Validation

- `cargo test --manifest-path tandem/Cargo.toml tui::tests`: **107 passed** (including actual keyboard prompt -> native record tests for claim, block, resume, deliver, complete; invalid transition, missing input, cancellation, and rendered acceptance/blocker/progress/evidence assertions).
- `cargo build --manifest-path tandem/Cargo.toml --release`: passed.
- `cargo fmt --manifest-path tandem/Cargo.toml -- --check`: passed after formatting changed TUI files.
- Release ANSI inspection: launched the release TUI in Herdr pane `w4P:p2` and read `--format ansi`. Board rendered title, state tabs/counts, selected task row, and `a Actions · e Edit · f Filter · v Validate · b Epic Board · ? Help`. The action picker also rendered in the narrow preview pane. Temporal flicker/resize latency and taste criteria remain unverified.

## Reproducible preview

The disposable fixture was created by the native release CLI at:

`/tmp/tandem-task-15-preview`

Create/run commands:

```sh
rm -rf /tmp/tandem-task-15-preview
mkdir -p /tmp/tandem-task-15-preview
cd /tmp/tandem-task-15-preview
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem init --title 'Task 15 TUI workflow preview'
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem add task 'Preview durable workflow actions' --acceptance 'Claim, deliver, block, resume, and complete from the TUI' --acceptance 'Show acceptance, blockers, milestone progress, and evidence'
/home/ivan/.herdr/worktrees/tandem/worker-task-15-make-durable-task-and-milestone-work/tandem/target/release/tandem tui
```

## Consumer/version handoff

The consumer is the native Rust TUI in the same `tandem` `0.12.3` binary. No extension or Pi adapter changes are required. Task-13 should reconcile native deliver evidence validation before claiming complete parity.
