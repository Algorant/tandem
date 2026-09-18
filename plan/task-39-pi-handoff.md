# Task 39 explicit native flush handoff

> **Superseded (Task 40).** This document records the historical Task 39
> explicit-flush contract, including its amend/reconcile behavior and the
> `amended`/`consolidated` JSON fields. Those were removed. The forward-only
> flush and its host boundary wiring are documented in
> [`task-40-pi-handoff.md`](task-40-pi-handoff.md); this file is retained as
> history only.

## Purpose

Tandem 0.13.4 folds pending `.tandem/` state into an unpushed non-merge real
commit, but that logic only ran at root assignment lifecycle boundaries.
Intermediate `add`, `update`, rule, Epic, and Subtask writes could stay dirty
indefinitely, and an ordinary `git commit && git push` never invoked Tandem.

`tandem checkpoint` is the explicit native, idempotent flush for adapters and
commit/push workflows. It is the authoritative integration point for this job:
it invokes the existing 0.13.4 checkpointer directly and never fabricates an
Accord, record, or lifecycle transition. This document is the consumer contract
for that command. The lifecycle boundary contract remains in
[`task-14-pi-handoff.md`](task-14-pi-handoff.md); Task 14's cutover historically
had no checkpoint-only command and that statement stays accurate.

## Command

```sh
tandem checkpoint
tandem --json checkpoint
```

`checkpoint` is a bare leaf. It takes no arguments and uses the global
`-j/--json` switch. It discovers the owning `.tandem/` workspace from the
current directory exactly like other commands and requires protocol `0.3.0`.

## Adapter handoff

The supported order is:

1. create the real source commit (leave `.tandem/` as the only pending state);
2. run `tandem checkpoint`;
3. require a clean `.tandem/` (a successful `checkpointed` or `clean` result
   already guarantees this because the flush stages and commits the owning
   path);
4. push.

`tandem checkpoint` therefore composes with shell fail-fast:

```sh
git commit -m 'feat: real source change'
tandem checkpoint
git push
```

There is no Git hook, push wrapper, or manual staging step. Adapters must not
commit, amend, rebase, or tidy `.tandem/` themselves; native Tandem is the sole
Git writer.

## Behavior

- With an unpushed non-merge real HEAD, the flush amends that commit with the
  `.tandem/` pathspec and preserves the real commit subject, so pending
  Task/update/rule/Subtask metadata rides in the last real commit.
- With an unpushed own checkpoint commit as HEAD, it amends that commit under
  the fixed subject `chore(tandem): checkpoint metadata`.
- With no absorbable real commit (pushed HEAD, merge HEAD, or no HEAD), it
  creates at most one rolling Tandem checkpoint rather than one commit per
  mutation.
- Pushed and merge commits are never amended. Unrelated staged, unstaged, and
  untracked files and index entries are never touched.
- On a clean `.tandem/` the command is idempotent: it reports `clean`, creates
  no commit, and still runs the leftover-chore reconcile.
- Eligible leftover own-chore runs are folded with the same safety rules as a
  lifecycle boundary (clean tree, `@{upstream}` ancestor of HEAD, no in-flight
  operation, lock held, best-effort).
- The flush writes no Task, Accord, Rule, Decision, or event bytes. It only
  stages and commits the owning `.tandem/` path through the existing repository
  lock.
- Unlike a lifecycle mutation, a checkpoint failure fails closed. A lifecycle
  command exits `0` because its record write succeeded; the standalone command
  has no successful record operation to preserve, so `tandem checkpoint &&
  git push` cannot proceed on failure.

## JSON contract

A flush that creates or amends a commit:

```json
{
  "ok": true,
  "data": {
    "checkpoint": {
      "status": "checkpointed",
      "commit": "<HEAD sha>",
      "amended": true,
      "consolidated": 0
    }
  },
  "warnings": []
}
```

The idempotent no-op on a clean `.tandem/`:

```json
{
  "ok": true,
  "data": {
    "checkpoint": {
      "status": "clean",
      "commit": null,
      "amended": false,
      "consolidated": 0
    }
  },
  "warnings": []
}
```

`clean` always reports `commit: null` and `amended: false` and creates no
commit. `checkpointed` always carries a non-null `commit`; `amended` is `true`
only when this flush amended its own unpushed HEAD (a pushed HEAD, a merge
HEAD, or no HEAD creates a new commit and reports `amended: false`).
`consolidated` is the number of adjacent own-checkpoint runs collapsed or
folded (`0` when nothing was collapsed or the rewrite was unsafe).

Failure (non-Git workspace, missing Git identity, failing commit hook, and
other Git failures):

```json
{
  "ok": false,
  "error": {
    "code": "checkpoint",
    "message": "<checkpoint error>",
    "details": {
      "checkpoint": {
        "status": "failed",
        "commit": null,
        "amended": false,
        "consolidated": 0,
        "error": "<checkpoint error>"
      }
    }
  }
}
```

The failure envelope is printed to stdout in JSON mode and the process exits
`1`. Human mode reports the failure on stderr and exits `1`. A failed
checkpoint leaves the pending `.tandem/` change staged for inspection or a
later retry; it never rolls back real record bytes.

## Evidence

`cargo test --manifest-path tandem/Cargo.toml --test checkpoint_behavior` uses
disposable real Git repositories and the actual CLI binary. The explicit-flush
tests prove:

- an ordinary unpushed source commit is amended with pending Task, update,
  Rule, and direct Subtask metadata, keeps its subject and parent, leaves
  `.tandem/` clean, and leaves unrelated staged, unstaged, and untracked state
  and index entries unchanged; the pending Rule and Subtask bytes are compared
  byte-for-byte against the committed blobs;
- the flush authors no Task, Decision, Rule, or event bytes;
- repeated flushes are idempotent on a clean `.tandem/`, and metadata-only work
  keeps at most one rolling checkpoint commit (unpushed amend and pushed
  create-then-amend);
- an unpushed merge HEAD is never amended;
- non-Git and failing-hook runs exit `1` with the exact
  `ok:false`/`code:checkpoint`/`details.checkpoint.status:failed` envelope,
  stdout-only in JSON mode, and leave `.tandem/` staged;
- eligible leftover own-chore runs fold into the adjacent real commit.

## Release boundary

A source build containing Task 39 is for development verification only: check
that `tandem --help` lists `checkpoint`, or run the JSON probe above in a
disposable Git repository. Pi readiness requires published Tandem 0.13.5 to be
available through the normal mise source; Pi must not implement against
unreleased main or a locally copied binary. Version
metadata, release notes, annotated tags, pushed tags, GitHub Release
publication, and release automation are owned by the Tandem orchestrator, not
this Task. No Pi adapter file, external Pi configuration, Git hook, or push
wrapper is added here.
