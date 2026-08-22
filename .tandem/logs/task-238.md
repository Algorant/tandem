---
id: task-238
type: task
title: "Enforce the ignored actor-id invariant and reject stale-state writes"
priority: "high"
effort: "small"
references: ["decision-9", "decision-10"]
relatedFiles: ["tandem/src/project/events.rs", "tandem/src/project/write.rs", "tandem/src/protocol/ids.rs", "protocol/README.md"]
tags: ["protocol", "events", "identity", "git", "bugfix"]
createdAt: "2026-08-22T14:39:15Z"
updatedAt: "2026-08-22T23:07:50Z"
accord:
  status: "accepted"
  assignee: "worker-task-238-93e9e45b"
  claimedAt: "2026-08-22T22:55:20Z"
  deliveredAt: "2026-08-22T23:07:30Z"
  validation:
    commands: ["cargo fmt", "cargo clippy --all-targets -- -D warnings", "cargo test: 277 unit + 11 integration passed (rerun independently by orchestrator)"]
  summary: "`ensure_git_ignored()` now hard-fails when `.tandem/actor-id` is tracked, naming `git rm --cached` and the `<actor>:<seq>` uniqueness it breaks, and re-verifies check-ignore after writing the exclude pattern so a .gitignore negation cannot silently defeat it. Regression test covers the tracked case. protocol/README.md states the invariant is enforced."
  filesChanged: ["tandem/src/project/events.rs", "protocol/README.md"]
  note: "Orchestrator-verified. Reran cargo test and cargo clippy --all-targets -- -D warnings in the Worker checkout before merge. All six acceptance criteria met, including the tracked-file regression test that check-ignore alone cannot detect. Confirmed this repository's own .tandem/actor-id is untracked and ignored, so the new hard failure does not self-trigger."
  updatedAt: "2026-08-22T23:07:44Z"
assignee: "worker-task-238-93e9e45b"
completedAt: "2026-08-22T23:07:50Z"
completion:
  summary: "`ensure_git_ignored()` hard-fails on a tracked `.tandem/actor-id` naming `git rm --cached`, and re-verifies check-ignore after writing the exclude pattern so a .gitignore negation cannot silently defeat it. Regression test added; protocol/README.md states the invariant is enforced. Merged to main as 98914a4."
---

## Description

## Problem

`decision-9` and `protocol/README.md` (lines 22, 77) both require `.tandem/actor-id` to be **ignored**, per independent checkout or linked worktree. Nothing enforces this. When the file is tracked and committed, every clone inherits one identity, every machine appends to one ledger, and each independently allocates the same `seq`. The result is a guaranteed merge conflict on every parallel session.

Observed in the wild in `~/.dotfiles` (`pi/.pi/.tandem/`): actor `07d56b30-3178-4ae3-99ef-8e962ae045c2` was committed, two machines both wrote `seq` 874-882, and the collision only surfaced as a rebase conflict. Manual repair required renumbering events and reassigning a document ID.

## Root cause

`ensure_git_ignored()` in `tandem/src/project/events.rs` runs on **every** `actor_id()` call, so frequency is not the issue. It asks the wrong question.

It calls `git check-ignore` and, on exit 1, appends the pattern to `.git/info/exclude`. But `check-ignore` never reports a **tracked** file as ignored, and ignore rules never apply to tracked files. So Tandem appended an already-present pattern, saw success, and concluded it was correct. Forever. The broken state is indistinguishable from the healthy one via `check-ignore` alone.

A repo `.gitignore` can also re-include the path and override `.git/info/exclude`, since `.gitignore` outranks `info/exclude`. In the observed case a `!pi/.pi/.tandem/**` negation did exactly that.

## Fix

In `ensure_git_ignored()`, beside the existing `check-ignore` shell-out, add a tracked check:

```
git -C <git_root> ls-files --error-unmatch <relative>
```

Exit 0 means tracked, which is the corrupt state. Fail with a `CliError::user` naming the remedy (`git rm --cached <path>`) and the reason (shared identity collapses per-actor ledgers). Exit 1 means untracked and correct.

This must be a hard failure, not a warning. A shared actor ledger silently violates the `<actor>:<seq>` uniqueness that `next_sequence()` depends on, and the damage is only visible later at merge time.

Also verify the pattern actually takes effect after writing it, rather than assuming. A negation in a tracked `.gitignore` can defeat `info/exclude`, so re-run `check-ignore` after appending and fail if the path is still not ignored.

## Related weakness, same root cause

`next_sequence()` (`events.rs`) and `next_sequential_number()` (`protocol/ids.rs`) both derive the next value as `max + 1` over what is currently on disk. No counters, deliberately, since a counter file is its own conflict hotspot.

That is correct for concurrent processes in one checkout, where `flock` and `write_new_atomic`'s `O_EXCL` probe serialize allocation. It has no defense across machines, because the other machine's files are not present locally when allocation happens. Two offline machines both allocate `task-N`, producing an add/add conflict. This also happened in the observed case (`task-249`).

The single invariant covering both collision points: **never write from stale state.**

Optional, lower value: before allocating an ID, if the branch has an upstream, check `git rev-list --count HEAD..@{u}` and refuse when non-zero. Note the real limit before implementing: without a fetch, `@{u}` is itself stale, so this only catches "fetched but not merged" and will not stop a genuinely offline second writer. Do not make Tandem fetch on every command.

## Explicitly out of scope

Per-machine ID prefixes, reserved ID ranges, UUID document IDs, and lease/holder files. Each trades readable `task-N` IDs or adds coordination to prevent a case that git already surfaces loudly as an add/add conflict. Duplicate document IDs are rare and self-announcing; the actor-id defect is silent and permanent, which is why only it warrants enforcement.

## Acceptance criteria

- A tracked `.tandem/actor-id` produces an immediate, actionable hard error naming `git rm --cached`.
- An untracked `.tandem/actor-id` in a Git checkout continues to work with no new output.
- A workspace outside any Git worktree is unaffected.
- An `actor-id` still not ignored after the exclude pattern is written (for example, via a `.gitignore` negation) is reported rather than silently accepted.
- Regression test covers the tracked-file case, which `check-ignore` alone cannot detect.
- `protocol/README.md` states that the ignored requirement is enforced, not merely expected.
