---
id: task-34
type: task
title: "Collapse unpushed tandem-only checkpoints natively"
state: "in-progress"
priority: "high"
effort: "medium"
references: ["task-14"]
relatedFiles: ["tandem/src/project/checkpoint.rs", "tandem/src/cli/commands.rs", "tandem/src/app/support.rs", "tandem/tests/checkpoint_behavior.rs", "protocol/README.md", "plan/task-14-pi-handoff.md", "scripts/tidy_history.sh", "justfile"]
tags: ["protocol", "git"]
accord:
  status: "claimed"
  acceptance: ["Protocol states the history invariant: unpushed adjacent `.tandem/`-only commits are not part of the intended shape; Tandem may rewrite its own unpushed tandem-only commits; pushed commits and ordinary/unproven work commits are never amended; a `meta / source / meta` sandwich is allowed.", "Assignment-boundary checkpoint amends HEAD when it is unpushed and tandem-only; otherwise it creates a new ordinary commit. Unrelated dirty index/worktree/untracked bytes are preserved. JSON `amended` is true only when an amend happened.", "When rewrite is safe (clean tree, upstream is an ancestor of HEAD, no in-progress rebase/merge/cherry-pick/revert, lock held), the same checkpoint also collapses any remaining adjacent tandem-only run in `@{upstream}..HEAD`. When unsafe, skip reconcile and do not fail the record write.", "Real-Git tests prove: repeated assignment boundaries on a dirty unrelated tree produce one rolling checkpoint, not N; an ordinary local commit and a pushed HEAD remain `HEAD^`; a leftover adjacent chore run is collapsed only on a safe boundary; the checkpoint lock still serializes.", "Consumer handoff: adapters must not checkpoint or tidy; they must accept `amended: true`. No Pi/adapter implementation in this Task. `just tidy-history` is removed or marked superseded so ordinary Tandem + git push is the supported workflow."]
  claimedAt: "2026-09-16T15:46:25Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml --test checkpoint_behavior", "$ cargo test --manifest-path tandem/Cargo.toml", "$ git diff --check", "Disposable real-Git probes: rolling amend on a dirty unrelated tree; append after ordinary and after pushed HEAD; reconcile of a leftover adjacent chore run only when rewrite is safe; lock still serializes.", "Inspect protocol README and lifecycle JSON: `amended` is true only when an amend happened; adapters are told not to checkpoint or tidy."]
  constraints: ["One Git writer: native Tandem. Do not modify `extensions/pi-tandem/` or any Pi/adapter implementation; publish a consumer handoff instead.", "Never rewrite pushed/shared commits or amend an ordinary/unproven work commit. No force, hook bypass, or second checkpointer.", "Do not fold tandem metadata into a neighboring source commit.", "No new user-facing just recipe, git hook, or Pi helper; workflow stays ordinary Tandem + git push.", "Tests may mutate disposable repositories only. Do not checkpoint this checkout, another worktree, or user Git config as an experiment."]
  updatedAt: "2026-09-16T15:46:25Z"
createdAt: "2026-09-16T15:22:34Z"
updatedAt: "2026-09-16T15:46:25Z"
assignee: "worker-task-34-55be0a8a"
---
Native assignment checkpoints always create a new `chore(tandem): checkpoint metadata` commit and never amend, including their own previous unpushed tandem-only HEAD. That last part is stricter than the real safety rule (do not amend pushed or ordinary commits). It floods `main`: a typical Pi session push was 9 commits, 8 of them chores; `~/.pi` last 200 commits were 160 chores. The same shape exists on this repo.

`just tidy-history` / `scripts/tidy_history.sh` (historical task-240) is a pre-push rebase workaround. It is not the product. Algorant wants this in the protocol and native checkpoint path, with no extra files, hooks, or Pi setup. Pi must remain a Tandem client, not a second Git writer.

## Protocol

Invariant: unpushed history must not contain adjacent tandem-only commits. Tandem may rewrite **its own** unpushed `.tandem/`-only commits to keep that true. Sandwich `meta / source / meta` stays; folding metadata into a source commit is out of scope.

Hard safety, unchanged: never amend a pushed commit; never amend an ordinary/unproven work commit; stage/commit only the owning `.tandem/` path; preserve unrelated staged/unstaged/untracked state; serialize via `tandem-checkpoint.lock`; record write stays successful if Git fails.

## Implementation split

1. **Live maintenance (primary).** At each assignment checkpoint, if HEAD is unpushed and tandem-only, `git commit --amend` the `.tandem/` pathspec. Otherwise create a new ordinary commit as today. This must work with unrelated dirty files, same as current checkpoint. JSON `amended` is `true` only for that amend (it is no longer a constant false safety assertion).

2. **Reconcile (backup).** After that, if rewrite is safe (clean tree, `@{upstream}` is an ancestor of HEAD, no rebase/merge/cherry-pick/revert in flight, lock held), collapse any remaining adjacent tandem-only run in `@{upstream}..HEAD`. If rewrite is not safe, skip; do not fail the record write. This repairs leftover runs from old binaries, failed amends, or pre-change history. It must not be a user-run recipe.

3. **Supersede `just tidy-history`.** Remove or clearly mark the script/recipe as superseded so the supported path is ordinary lifecycle + `git push`.

## Consumer handoff (no adapter work here)

Adapters must not commit, amend, rebase, or tidy `.tandem/`. They must accept `checkpoint.amended: true` and any consolidate result without replaying the lifecycle mutation. Pi `~/.pi` task-146/task-98 captured the research; do not implement Pi-side tidy.

## Non-goals

Jujutsu. Mixed code+`.tandem` commits. Rewriting origin. Restoring Pi housekeeping. Per-task grouping of checkpoints.