---
id: task-48
type: task
title: "Review and redesign cross-machine sync: collision-free IDs, conflict-free commits/pushes, and an automated reconcile flow"
state: todo
priority: "high"
effort: "large"
relatedFiles: [".tandem/decisions/decision-7.md", "docs/workspace/index.md", "docs/concepts/index.md", "docs/guides/agents-and-adapters.md"]
tags: ["sync", "ids", "conflicts", "reconcile", "checkpoint", "multi-machine"]
accord:
  status: "ready"
  acceptance: ["A written review documents how Tandem currently allocates IDs, writes events/records, checkpoints, consolidates, and expects pull/push to work across machines, and names every failure mode observed in real use (at minimum: sequential ID collisions from behind checkouts, untracked records blocking `git pull --ff-only`, duplicate-document-ID errors after pull, and text merge conflicts in `.tandem/`), each with root cause verified against code rather than assumed.", "The review explains the gap between the intended 'machine/actor-specific identity' model (per-actor UUID event ledgers, decision-7) and the actual behavior (globally sequential task/decision IDs allocated from local state), and states what that identity model does and does not protect.", "A recommended design makes routine multi-machine use conflict-free by construction: ID allocation that cannot collide across machines/checkouts (or is safely provisional until sync), record and event storage that Git merges without manual conflicts, and checkpoint/consolidate/push behavior that never leaves a checkout unable to fast-forward. Alternatives considered are listed with reasons for rejection.", "A reconcile/conflict flow is specified that runs automatically on pull/checkout reconcile for the common cases (e.g. unstarted, unpushed local record colliding with an upstream ID is refiled to a fresh ID with references and events rewritten, and old→new IDs reported), and stops with a precise diagnostic naming both sides only for genuinely ambiguous cases (both claimed/delivered/logged, both committed, divergent edits to the same record). Agents and Pi never hand-edit `.tandem/` to resolve it.", "A migration path for existing workspaces (~/.pi, ~/.dotfiles, this repo) is defined so existing IDs, references, and logs stay valid.", "Algorant reviews and approves the design before implementation tasks are created; implementation is split into follow-up Tasks/subtasks only after that approval."]
  validation: ["Reproduce each failure mode with two independent clones of a scratch workspace (simulating two machines): create records on both while one is behind, then pull/checkpoint/consolidate/push, and record what breaks today.", "Show the proposed design handles the same two-clone scenarios with zero manual conflict resolution in the common cases and a clear stop in the ambiguous ones.", "Check the design against decision-7 (worktree-local actor identity) and decision-8 (protocol 0.3.0) and state whether either must be superseded."]
  constraints: ["Review and design first; do not change the protocol, ID format, or on-disk layout until Algorant approves the design.", "No Pi-side or adapter-side fallback that deletes, renames, or hand-edits `.tandem/` files; detection, refile, and merge behavior belong in Tandem.", "Do not create an Epic unless Algorant approves one; use subtasks/milestones within this Task if steps are needed."]
  updatedAt: "2026-09-29T18:30:34Z"
createdAt: "2026-09-29T18:30:34Z"
updatedAt: "2026-09-29T18:30:34Z"
---

## Description

## Why

Algorant, 2026-09-29: syncing Tandem metadata between machines (desktop, algotop, archbox, work, omarchy/x1nano) produces conflicts almost every time. These are plain text files; routine sync should not require manual repair. The expectation was that machine-specific IDs would make most of this reconcile automatically, and that remaining conflicts would have a mostly automated resolution flow. Neither is true today.

## Observed failures (all on desktop)

- **2026-09-24, Algorant/.pi:** local task-265 (+ subtasks) and task-266 collided with different upstream task-265/266. Untracked `logs/task-265.md` blocked the rebase; repaired by a hand-written user-run renumbering script. Tracked as Algorant/.pi task-272.
- **2026-09-27, Algorant/.pi:** local untracked task-273 collided with an archived upstream task-273; `tandem show` failed with a duplicate document ID until a person authorized deleting the local file. Proposed Tandem-side fix filed as Algorant/.pi task-275 (refile unstarted duplicate IDs during reconcile).
- **2026-09-29, Algorant/.pi:** unpushed local task-274..277 (created Sep 25–28) collided with upstream task-274 (logged) and task-275. `sysup pi` (`git pull --ff-only`) refused: "Untracked working tree file '.tandem/tasks/task-275.md' would be overwritten by merge". Required a user-run set-aside script, then manual re-creation with `tandem add` as task-276..279 and hand-fixed cross-references.
- **2026-09-29, Algorant/.dotfiles:** an agent created task-48 while the checkout was 89 commits behind; upstream already had a different task-48. Same set-aside/refile cycle (now task-49).

## Current model as understood (to be verified)

- Event ledgers are per-actor: `.tandem/events/<actor-uuid>.jsonl`, with the UUID worktree-local and uncommitted in `.tandem/actor-id` (decision-7). This avoids concurrent appends to one file, but an actor ledger still receives local appends that can diverge.
- Task/decision IDs are globally sequential (`task-N`) and allocated from the local checkout's view, with no fetch or behind-upstream check, so any two machines that create records between syncs collide.
- `checkpoint` flushes forward-only commits; `checkpoint --consolidate` squashes eligible unpushed metadata commits at push time. Unpushed records stay untracked or uncommitted in the working tree between sessions, which is exactly the state that blocks `pull --ff-only`.
- No `.gitattributes` merge strategy (e.g. union for append-only ledgers) is declared in consuming repos.
- In Algorant/.dotfiles, mise's automatic dotfiles history sync also commits and merges into the same branch every few minutes, which adds merge traffic around `.tandem/` commits.

## Directions to evaluate (not decisions)

- Collision-free IDs: actor/machine-scoped or random/ULID-style IDs with a short display alias; or provisional local IDs promoted on sync; or fetch-and-reserve before allocation.
- Storage that Git merges cleanly: one file per record plus append-only per-actor ledgers with union merge; avoid shared mutable index files.
- Commit timing: commit (not leave untracked) new records immediately so pulls rebase/merge instead of refusing; decide how that interacts with consolidation.
- An automatic `tandem reconcile` (or pull hook) implementing the refile/stop rules from Algorant/.pi task-275.
- Push/pull guidance for agents and `sysup` so a behind checkout is fixed before Tandem allocates anything.
