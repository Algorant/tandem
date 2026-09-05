# Task 15 Pi handoff

## Version and scope

- Final validation ran on Tandem `0.12.3` after Task-13 native evidence validation (`a7a60dc`) was integrated on main `361345f`.
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

- Full current `cargo test --manifest-path tandem/Cargo.toml`: **254 unit + 1 accord integration + 5 assignment integration + 6 CLI integration passed**.
- Native evidence cases passed in the same run: protocol rejects empty/blank evidence; app and CLI reject it without changing records/events; successful evidence remains observable.
- TUI action tests passed for comma-preserving prose, empty/whitespace/comma-only evidence, missing assignee/note, cancellation, stale-record native error retention, keyboard and mouse picker/modal paths, role-correct Task/Subtask rendering, archived completed/canceled/failed outcomes, long detail scrolling, final log evidence, and the rendered `Editing evidence (commas preserved)` prompt label.
- `cargo build --manifest-path tandem/Cargo.toml --release`: passed on the integrated source.
- `cargo fmt --manifest-path tandem/Cargo.toml -- --check`: passed; `git diff --check`: passed.
- Release ANSI inspection: launched the final release TUI against the enriched fixture in Herdr pane `w4P:p3` (zoomed to 140 columns) and read `--format ansi`. It rendered the selected delivered Task, active milestone child, state counts, and the `Task actions` picker with Claim, Deliver, and Complete options. Temporal flicker/resize latency remains unmeasured; no human-review gate is requested.

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

The consumer is the native Rust TUI in the same `tandem` `0.12.3` binary. No extension or Pi adapter changes are required. Native delivery validation is now supplied by Task-13 and consumed directly by the TUI prompt.
