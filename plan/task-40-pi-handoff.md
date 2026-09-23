# Task 40 host commit/push boundary handoff

## Purpose

Task 40 removed automatic Git activity from native Tandem. Task, record, and
event mutations now persist immediately and never stage, commit, amend, rebase,
or reconcile anything. Plain `tandem checkpoint` is the forward-only native
flush that collects pending owning `.tandem/` changes into one ordinary commit.

Making that flush automatic at a commit/push boundary is **host adapter work**,
not native work. This document is the consumer contract and the explicit
follow-up handoff for that adapter wiring.

## Prerequisite status

Native Task 40 (forward-only checkpointer, immediate lifecycle persistence,
updated protocol/CLI guidance) is a prerequisite, not end-to-end completion.
The end-to-end requirement is unmet until a separate adapter Task:

1. wires the boundary call sequence below into the real commit/push path, and
2. verifies it against an installed Tandem release that contains Task 40.

Do not claim end-to-end "metadata is collected automatically at commit/push"
from native-only evidence. Automatic batching is a host responsibility; it must
not be reimplemented as a per-mutation native commit and must not be deferred
to a manual command the user is expected to remember.

## Exact adapter ownership and discovery

No file in `extensions/pi-tandem/`, no external Pi configuration, and no Git
hook is changed by Tasks 40 or 43. An adapter Task owns:

- `extensions/pi-tandem/index.ts` — the Pi tool/command layer and its
  `execFile` wrapper. Any new `tandem_checkpoint` tool, commit/push command
  hook, or boundary call belongs here.
- `extensions/pi-tandem/pi-tandem.md` — agent-facing guidance for when the
  boundary runs.
- `extensions/pi-tandem/plan/todo.md` — the adapter prerequisite checkbox and
  its integration evidence.
- The actual commit/push integration point, wherever the host initiates
  commits and pushes. Discover it before editing; do not assume a location:
  - search the repo adapter and shared Pi config for `git commit`, `git push`,
    and `checkpoint` (for example
    `rg -n "git (commit|push)|checkpoint" extensions/pi-tandem ~/.pi/agent`);
  - inspect any project or user Git `pre-commit`, `pre-push`, or `commit-msg`
    hooks, and any Pi command/alias wrapper around `git`;
  - reuse the project-local loader pattern in
    `extensions/pi-tandem/tests/pi-runtime-smoke.ts` when validating a
    project-local `.pi/extensions/pi-tandem/index.ts` registration.

If no single commit/push integration point exists, the adapter must add one
(an explicit Pi command hook or documented wrapper). It must not add a
best-effort background reconciler, a second Git writer, or a branch
reachability heuristic.

## Native contract the adapter consumes

Every lifecycle mutation (claim, deliver, rework, block, resume, release, fail,
review, complete, cancel) returns JSON with:

```json
{"ok":true,"data":{"id":"task-1","recordWritten":true,"checkpoint":{"status":"batched","commit":null}},"warnings":[]}
```

`checkpoint.status: "batched"` is a truthful pending-metadata signal. It means
the record is durable in the worktree and no commit was made. It is never a
request to commit immediately and never carries a rewrite outcome; only the
explicit push-boundary command can report `status: "consolidated"`.

The flush command:

```sh
tandem --json checkpoint
```

returns either

```json
{"ok":true,"data":{"checkpoint":{"status":"checkpointed","commit":"<sha>"}},"warnings":[]}
```

or, when nothing is pending,

```json
{"ok":true,"data":{"checkpoint":{"status":"clean","commit":null}},"warnings":[]}
```

The plain flush stages only the owning `.tandem/` path, appends at most one ordinary
`chore(tandem): checkpoint metadata` commit, never rewrites an existing commit,
and preserves unrelated staged entries, unstaged bytes, and untracked files.
It writes no Task, Accord, Rule, Decision, or event bytes.

## Commit boundary call sequence

1. Let the host create the real source commit first, so `.tandem/` is the only
   remaining pending state. A metadata-only boundary may also run the flush
   with no source commit; do not fabricate an empty source commit.
2. Run `tandem --json checkpoint` from the workspace root.
3. Parse the envelope:
   - `status: "checkpointed"` — one forward metadata commit was appended; the
     `.tandem/` worktree is clean. Continue.
   - `status: "clean"` — nothing was pending; no commit was created. Continue.
4. If the command fails, treat the boundary as failed (see below).

## Push boundary call sequence

1. Run `tandem --json checkpoint --consolidate` **only** at the push boundary.
   This first runs the forward-only flush (including metadata-only work), then
   consolidates eligible unpushed checkpoints. It rewrites the IDs of unpushed
   real commits while preserving their order, messages and final tree. Never
   use this option at commit boundaries or on lifecycle mutations.
2. Continue only on a successful `status: "consolidated"` response, and only
   when `git status --porcelain -- .tandem` is empty.
3. Push. Do not replay a failed push-boundary operation with an alternate Git
   writer or force-push.

The fail-closed composition is:

```sh
tandem checkpoint --consolidate && git push
```

The response includes `oldHead`, `newHead`, `commit` (the new HEAD), and
`collapsed` (the number of eligible metadata-only commits). Zero is a clean
no-op. Consolidation requires an upstream ancestor and a clean owning
`.tandem/` path after the flush. It preserves unrelated staged entries,
unstaged edits, and untracked files, even on a dirty Worktrunk target: replay
uses a private index and never checks out the rewritten commits. It refuses
with a checkpoint error envelope without rewriting for an in-progress Git
operation, merge commit, real commit touching `.tandem/`, signed commit, or
any other local branch/linked worktree based inside the rewritten range. If
refusal occurs after a pending flush, that forward-only commit remains; the
adapter must stop before pushing. The default `checkpoint` command remains
the forward-only commit-boundary operation.

## Clean/no-op handling

For the plain flush, `clean` is success, not an error. It creates no commit, does not amend, and
does not rewrite history. It also does not touch unrelated dirt: a dirty
working tree outside `.tandem/` is left exactly as it was. Adapters must not
turn a clean flush into an empty commit or a retry.

## Failure behavior

A failed flush exits `1` and, in JSON mode, prints the failure on stdout:

```json
{"ok":false,"error":{"code":"checkpoint","message":"...","details":{"checkpoint":{"status":"failed","commit":null,"error":"..."}}}}
```

Human mode writes the message to stderr and exits `1`. On failure the pending
`.tandem/` change is left staged for inspection. The adapter must:

- abort the commit/push boundary and surface the message;
- never auto-resolve, force, stash away, skip, or retry with an alternate
  strategy;
- not treat the successful `recordWritten: true` in a prior mutation as
  permission to push dirty metadata.

## Negative controls the adapter must preserve

- A genuine overlapping `.tandem` edit conflict is reported as a conflict and
  left for a human; there is no merge driver, ours/theirs winner, or automatic
  repair.
- Published/shared history is never rewritten; the explicit push-boundary
  mode may rewrite only eligible unpushed commits after its safety checks.
- The adapter never commits, amends, rebases, or tidies `.tandem/`
  itself; Tandem remains the sole metadata Git writer.

## Evidence

Native evidence lives in
`tandem/tests/checkpoint_behavior.rs` and is run with
`cargo test --manifest-path tandem/Cargo.toml --test checkpoint_behavior`. It
uses disposable real Git repositories, the candidate binary, and installed
Worktrunk to prove pending tracked/untracked/deleted target metadata survives a
source-only worker merge with stable base commit IDs, that one later flush
appends and a repeat flush is clean, that a failing commit hook fails closed,
and that overlapping metadata conflicts are reported without suppression.
