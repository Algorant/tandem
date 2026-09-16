# Task 14 native checkpoint handoff

## Verified availability

Integrated on local `main` at `937a6207202a710dea4aed884e53e8633ff5e381`.
The orchestrator independently reran the complete merged suite (258 unit,
1 Accord, 5 assignment, 9 checkpoint, and 6 CLI tests), the real-Git owner
probes, and actual release ANSI failure rendering. Normal and exceptional
Validation tests show durable-write success and checkpoint failure at 80/120
columns. The installed binary is unchanged; Pi adapter removal remains a
separate coordinated cutover, not part of this source delivery.

## Native policy

The Rust application writes the Tandem record and its actor event immediately.
It calls the native Git checkpoint only after these writes at a lifecycle
boundary. Intermediate `update`, `move`, and other progress/metadata writes do
not commit.

The boundary set for an assignment is:

- `accord claim`: root Task or direct Epic Task work start.
- `accord deliver`: delivery.
- `accord block`: pause/block.
- `complete`: routine finish/archive.
- `accord fail` and `cancel`: terminal finish/archive.
- `accord resume`, `accord rework`, and `accord release`: lifecycle changes
  also checkpoint because they reopen or relinquish an active agreement.
- `review` request, Validation rework, and Validation acceptance use the same
  app-owned checkpoint path. Acceptance archives the Task and is a finish.

Native resolved hierarchy roles determine ownership. A root Task and a direct
Task beneath an Epic are assignments and checkpoint all changed `.tandem`
content at those boundaries. An Epic is grouping-only and a direct Subtask is
an assignment milestone. Epic and Subtask lifecycle writes, including claim,
deliver, block, resume, rework, release, fail, cancel, complete, and any
allowed validation action, return `checkpoint.status=batched`: their record and
event writes persist immediately, but no Git commit is made. The next actual
assignment boundary captures them with the owning `.tandem` path. IDs are never
used to infer this role. There is no daemon, attempt history, per-edit commit,
or forced history rewrite.

The checkpoint stages and commits only the discovered project's `.tandem/`
path with the fixed subject `chore(tandem): checkpoint metadata`. It never
stashes, resets, cleans, changes unrelated index entries, or touches unrelated
worktree/untracked bytes. The lock file is `tandem-checkpoint.lock` in Git's
common directory, so linked worktrees and independent processes serialize the
Git add/commit sequence. Application hierarchy/write locks are released before
Git runs hooks; the coherent native result is retained in memory while the
checkpoint runs. The lock is outside the worktree and is not tracked.
`.tandem/actor-id` and other ignored runtime files remain uncommitted.

An assignment boundary keeps unpushed history from accumulating adjacent
Tandem-only commits. When HEAD exists, is unpushed (not contained in any
remote-tracking ref), and is Tandem's own `.tandem`-only checkpoint commit,
the boundary amends that HEAD with the `.tandem/` pathspec; otherwise it
creates a new ordinary commit. Amending and folding are limited to Tandem's
own unpushed checkpoint commits. Pushed commits and ordinary/unproven work
commits are never rewritten, and metadata is never folded into a neighboring
source commit, so a `meta / source / meta` sandwich is preserved. Repeated
lifecycle actions that have no new `.tandem` diff report `clean` and create no
empty commit. The same boundary then reconciles every remaining adjacent run
of Tandem's own checkpoint commits inside `@{upstream}..HEAD`, collapsing each
run to one commit, but only when the tree is clean, `@{upstream}` is an
ancestor of HEAD, no rebase/merge/cherry-pick/revert is in flight, and the
checkpoint lock is held. Reconcile is best-effort: an unsafe or failed rewrite
is skipped and reported as `consolidated: 0`, and it never turns a successful
record write into a failure.

Non-Git projects, a workspace outside the Git root, an unsupported Git state,
Git add failure, and hook/commit failure are explicit `failed` checkpoint
results. There is no silent fallback or force/hook bypass. Nested workspace
discovery uses the Git repository containing the Tandem workspace; a workspace
outside that repository is rejected.

## Result contract

Every lifecycle mutation returns a successful native record result even when
its subsequent Git checkpoint fails. The result does not invite replay of the
successful action: recovery uses the next real assignment boundary, which
captures the still-staged durable write. There is no checkpoint-only CLI
command in this cutover. The result has:

```json
{
  "ok": true,
  "data": {
    "id": "task-1",
    "recordWritten": true,
    "checkpoint": {
      "status": "checkpointed|batched|clean|failed",
      "commit": "<HEAD sha>|null",
      "amended": false,
      "consolidated": 0,
      "error": "<message>"
    }
  },
  "warnings": []
}
```

`error` is present only for `failed`. `amended` is `true` only when this
boundary amended its own unpushed tandem-only HEAD, and `consolidated` is the
number of adjacent own-checkpoint runs collapsed by the best-effort reconcile
(`0` when nothing was collapsed or the rewrite was unsafe). A
failed checkpoint leaves the native `.tandem` change staged for inspection or a
later boundary; it does not roll back the durable record. Human CLI and TUI
surfaces say `record written` separately from `Git checkpoint FAILED`.

The CLI uses the source build containing this task:

```sh
cargo build --manifest-path tandem/Cargo.toml --release
cd /path/to/project
/path/to/tandem/target/release/tandem --json accord claim task-1 --assignee worker
/path/to/tandem/target/release/tandem --json accord deliver task-1 \
  --summary 'ready' --evidence 'observed output'
/path/to/tandem/target/release/tandem --json complete task-1
```

A successful assignment boundary has `checkpoint.status=checkpointed` and a
commit SHA. A milestone/grouping mutation has `batched`; a no-op assignment
boundary has `clean`. A hook failure has `failed` while
`recordWritten` remains true. The installed `tandem 0.12.3` binary is not
updated by this task; availability means a build containing the integrated
Task-14 commit.

## Evidence

`cargo test --manifest-path tandem/Cargo.toml --test checkpoint_behavior`
uses disposable real Git repositories and the actual CLI binary. It proves:

- partially staged, unstaged, and untracked unrelated bytes/index entries are
  unchanged after a claim;
- claim, intermediate metadata update, delivery, and completion produce only
  actual boundary commits, with no empty commit and no amendment;
- a genuine pre-commit hook failure reports `recordWritten=true` and a
  distinct `checkpoint.status=failed`, leaves the change staged, and the next
  assignment boundary captures it without replaying the original action;
- milestone claim, block, resume, delivery, metadata progress, and completion
  return `batched` with no HEAD change; a root boundary captures their logs and
  events. Epic grouping is batched while direct Epic Tasks checkpoint;
- staged additions, deletions, and partial index entries remain untouched;
  clean no-op checkpointing, non-Git failure, nested workspaces, linked
  worktrees, hook reads, and rendered TUI success/failure outcomes are covered;
- two independent CLI processes can checkpoint separate Tasks concurrently
  without corrupting the repository or Git object database.

The full Rust suite also passed on the corrected source:
`257` unit tests, `1` Accord integration test, `5` assignment integration
tests, `6` CLI integration tests, and the nine real-Git checkpoint tests.
`cargo build --manifest-path tandem/Cargo.toml --release` and
`git diff --check` are required final checks.

## Safe TUI preview

For owner ANSI inspection, use only this disposable fixture and the release
binary built from the integrated source:

```sh
BIN="$PWD/tandem/target/release/tandem"
FIXTURE=/tmp/tandem-task14-tui-preview
rm -rf "$FIXTURE"
mkdir -p "$FIXTURE"
cd "$FIXTURE"
git init --quiet
git config user.name 'Tandem TUI Preview'
git config user.email preview@example.invalid
"$BIN" init --title 'Task 14 TUI checkpoint preview'
"$BIN" add task 'Preview checkpoint outcome' --acceptance 'inspect native result'
git add .tandem && git commit --quiet -m baseline
"$BIN" tui
```

Press `a`, choose Claim, enter an assignee, and inspect the footer for
`record written; Git checkpointed`. To inspect failure rendering, exit the TUI,
install a disposable failing hook, and relaunch:

```sh
printf '#!/bin/sh\nexit 1\n' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
"$BIN" tui
```

The footer must retain the durable result and visibly distinguish `RECORD
WRITTEN but Git checkpoint FAILED`. Remove the fixture after inspection.

## Pi cutover

Pi must first verify native availability by building or invoking a binary that
contains this implementation, then run the JSON claim/delivery/finish probe in
a disposable Git repository and inspect `data.recordWritten` plus
`data.checkpoint.status`. Only after that probe succeeds should the Pi consumer
remove its `tandem-housekeeping.ts` registration and event emissions. It must
not keep both paths enabled: native lifecycle calls are the sole checkpoint
owner after cutover. No Pi adapter file is changed by Task 14.

During the transition, Pi may continue its old housekeeping only with a binary
that does not invoke native checkpointing. The cutover is therefore an
availability switch, not a retry or replay. Existing adapter reports and
`expectedHead`/amend logic are not part of the native contract. If native
availability cannot be verified, leave the adapter in place and do not claim
that the cutover is complete.

Adapters are consumers, never a second Git writer. They must not commit,
amend, rebase, or tidy `.tandem/`, and they must accept `checkpoint.amended:
true` and any consolidate result without replaying the lifecycle mutation that
produced it. Native Tandem owns the live amend and the reconcile, so adapters
must not run a pre-push history recipe. The historical `just tidy-history`
recipe and `scripts/tidy_history.sh` are removed: the supported workflow is
ordinary Tandem lifecycle calls plus `git push`.

The unresolved consumer choice is scheduling the adapter removal in a later
Pi-owned task. Native policy, JSON fields, failure semantics, and the exact
availability probe above are settled here; adapter retirement itself remains
outside core Task 14 ownership.
