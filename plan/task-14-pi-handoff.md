# Task 14 native checkpoint handoff

## Native policy

The Rust application writes the Tandem record and its actor event immediately.
It calls the native Git checkpoint only after these writes at a lifecycle
boundary. Intermediate `update`, `move`, and other progress/metadata writes do
not commit.

The boundary set is:

- `accord claim`: root work start.
- `accord deliver`: delivery.
- `accord block`: pause/block.
- `complete`: routine finish/archive.
- `accord fail` and `cancel`: terminal finish/archive.
- `accord resume`, `accord rework`, and `accord release`: lifecycle changes
  also checkpoint because they reopen or relinquish an active agreement.
- `review` request, Validation rework, and Validation acceptance use the same
  app-owned checkpoint path. Acceptance archives the Task and is a finish.

A Subtask and an Epic use the same policy as a root Task. There is no milestone
or root-specific commit path. There is no daemon, attempt history, per-edit
commit, or forced/amended commit.

The checkpoint stages and commits only the discovered project's `.tandem/`
path with the fixed subject `chore(tandem): checkpoint metadata`. It never
stashes, resets, cleans, changes unrelated index entries, or touches unrelated
worktree/untracked bytes. The lock file is `tandem-checkpoint.lock` in Git's
common directory, so linked worktrees and independent processes serialize the
Git add/commit sequence. The lock is outside the worktree and is not tracked.
`.tandem/actor-id` and other ignored runtime files remain uncommitted.

Native checkpoints always create a new ordinary commit. They do not amend
pushed, unpushed, ordinary, or previous checkpoint commits. Repeated lifecycle
actions that have no new `.tandem` diff report `clean` and create no empty
commit. A real boundary creates one commit, so history is bounded by actual
boundaries rather than by every progress edit.

Non-Git projects, a workspace outside the Git root, an unsupported Git state,
Git add failure, and hook/commit failure are explicit `failed` checkpoint
results. There is no silent fallback or force/hook bypass. Nested workspace
discovery uses the Git repository containing the Tandem workspace; a workspace
outside that repository is rejected.

## Result contract

Every lifecycle mutation returns a successful native record result even when
its subsequent Git checkpoint fails. This prevents an interface from inviting
a retry that repeats an already-successful mutation. The result has:

```json
{
  "ok": true,
  "data": {
    "id": "task-1",
    "recordWritten": true,
    "checkpoint": {
      "status": "checkpointed|clean|failed",
      "commit": "<HEAD sha>|null",
      "amended": false,
      "error": "<message>"
    }
  },
  "warnings": []
}
```

`error` is present only for `failed`. `amended` is always `false` in the
native implementation and is retained as an explicit safety assertion. A
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

A successful boundary has `checkpoint.status=checkpointed` and a commit SHA.
A repeated no-op boundary has `clean`. A hook failure has `failed` while
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
  distinct `checkpoint.status=failed`, leaves the change staged, and makes a
  retry fail the already-consumed lifecycle transition;
- two independent CLI processes can checkpoint separate Tasks concurrently
  without corrupting the repository or Git object database.

The full Rust suite also passed on the predecessor-integrated source:
`255` unit tests, `1` Accord integration test, `5` assignment integration
tests, `6` CLI integration tests, and the five real-Git checkpoint tests.
`cargo build --manifest-path tandem/Cargo.toml --release` and
`git diff --check` are required final checks.

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

The unresolved consumer choice is scheduling the adapter removal in a later
Pi-owned task. Native policy, JSON fields, failure semantics, and the exact
availability probe above are settled here; adapter retirement itself remains
outside core Task 14 ownership.
