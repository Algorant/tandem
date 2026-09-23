---
id: task-43
type: task
title: "Add an explicit push-boundary consolidation for unpushed checkpoint commits"
state: todo
priority: "medium"
effort: "medium"
references: ["task-40", "task-42"]
relatedFiles: ["protocol/README.md", "plan/task-40-pi-handoff.md"]
tags: ["checkpoint", "git", "workflow"]
accord:
  status: "ready"
  acceptance: ["`tandem checkpoint --consolidate` (or equivalent) collapses all fixed-subject .tandem-only checkpoint commits in @{upstream}..HEAD into one final checkpoint commit, replaying real commits in order; resulting HEAD tree equals the prior HEAD tree", "It refuses without rewriting when a linked worktree or other local branch is based inside the range, when a real commit in range touches .tandem/, when a merge commit is in range, when no upstream exists, or when a Git operation is in progress, with a checkpoint error envelope", "Default `tandem checkpoint` remains forward-only and unchanged; lifecycle mutations never consolidate", "Protocol README and host handoff document the push-boundary contract", "Demonstrated on a disposable repo reproducing an interleaved chore/real history plus a refusal case with a linked worktree"]
  validation: ["$ cd tandem && cargo test"]
  updatedAt: "2026-09-23T01:57:21Z"
createdAt: "2026-09-23T01:57:02Z"
updatedAt: "2026-09-23T01:57:21Z"
---

## Description

Requested by Algorant (2026-09-22) from the ~/.pi workspace.

## Problem
Since 0.13.6 (task-40) `tandem checkpoint` is forward-only: each flush appends one `chore(tandem): checkpoint metadata` commit. In a busy host repo, multiple flushes between pushes (other sessions, Worker lifecycle boundaries) leave several chore commits interleaved with real commits, e.g. ~/.pi unpushed range on 2026-09-22:

```
0550716 Keep worker active after target guard blocks
51cff8e feat(agency): support plan-first Worker Tasks
ae20b61 chore(tandem): checkpoint metadata
e9acb25 Capture Worker source snapshot at report time
e6dd69a chore(tandem): checkpoint metadata
863d5df fix(workers): block Worker tool access to orchestrator target
e21541e chore(tandem): checkpoint metadata
1ccfef1 fix(models): advertise claude-cli/2.1.280 ...
```

Pushed history on 09-21 shows 4 consecutive chore commits from 4 checkpoint+push cycles. Algorant wants a clean history with Tandem metadata combined into one commit per push, while keeping Worktrunk integration unaffected (pending `.tandem/` in the target is already tolerated by `wt merge`; confirmed live in ~/.pi).

## Requested design (explicit, not automatic)
Add an explicit push-boundary option (e.g. `tandem checkpoint --consolidate`) that:
1. Flushes pending `.tandem/` as today.
2. Within the unpushed range `@{upstream}..HEAD` only, rewrites history so every real commit is replayed in order and all `.tandem/`-only checkpoint commits collapse into one final `chore(tandem): checkpoint metadata` commit. Final tree must equal the pre-consolidation HEAD tree.
3. Is safe by construction: only commits whose diff is exclusively under the owning `.tandem/` and whose subject is the fixed checkpoint subject are collapsed; any real commit touching `.tandem/` aborts (no partial rewrite). Merge commits in range abort.
4. Refuses (clean error envelope, no rewrite) if any other local branch or linked worktree has its merge-base with HEAD inside the rewritten range, or if no upstream exists, or if a Git operation is in progress. Rationale: rewriting a base under a live Worker branch reintroduces the stale-base replay task-40 removed.
5. Never touches pushed commits; never runs automatically on lifecycle mutations. Reports old/new HEAD and collapsed commit count in JSON.

Keep the default forward-only `tandem checkpoint` behavior unchanged. Update protocol/README.md and the host handoff contract so adapters call the consolidating flush only at their push boundary.
