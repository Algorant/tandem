---
id: decision-9
uid: df435de0-bcba-4205-bdd2-42d03da87dbd
type: decision
title: "Sync the board through a tandem branch with permanent uids and numbering at first sync"
status: "accepted"
deciders: ["Algorant"]
references: ["task-48", "task-49", "decision-7", "decision-8"]
tags: ["protocol", "sync", "ids", "git", "multi-machine"]
createdAt: "2026-09-30T04:40:13Z"
updatedAt: "2026-09-30T04:40:26Z"
decidedAt: "2026-09-30T04:40:26Z"
---

## Status

Accepted by Algorant on 2026-09-30 (task-48). Implemented as protocol 0.4.0 in task-49.

## Context

Algorant works alone across several machines on the same repositories. Tandem metadata travelled inside source commits and new records took the next local number, so two machines that created records between syncs collided on IDs, untracked records blocked `git pull --ff-only`, archived and active copies of one ID broke every command, Markdown merged by line, and forward checkpoint commits diverged history and refused consolidation. Reproduction and root causes are in `protocol/plan/independent-metadata-sync.md`.

## Decision

- The board syncs through a Tandem-managed `tandem` branch on the repository's existing remote, independent of source commits. Source branches no longer track `.tandem/`.
- The board stays in the visible `.tandem/` folder of the main worktree, shared by linked worktrees, with a low-overhead local safety copy (`refs/tandem/pending`) that restores unsynced changes after old checkouts or `git clean`.
- Every record has a permanent `uid`. Human IDs stay sequential: a record created while it cannot be published gets a provisional `<prefix>-new-<hex>` ID and receives its number at first sync. Published IDs never change.
- Sync merges records semantically by `uid`; only contradictory edits, and archive on one machine with an edit on another, stop that one record and ask which version to keep.
- Direct Markdown edits are supported and validated; invalid edits stay local.
- Sync runs while Tandem is used (after mutations, on stale reads, and in the background of the TUI and web view); there is no daemon.
- `tandem migrate` converts 0.3.0 boards once; `tandem migrate --adopt` brings over unpushed legacy work from other machines.

## Consequences

- Checking out an old source commit no longer shows the board as it was; Tandem warns and keeps the current board safe.
- `tandem checkpoint` and consolidation are removed. Mutation results report `sync` instead of `checkpoint`.
- Pi (pi-tandem, agency, skills, prompts) and sysup need handoff updates before rolling out, because older guidance calls `checkpoint`.
- Protocol 0.4.0 boards are refused by older Tandem; 0.3.0 boards are refused until migrated.

## Supersession

Amends decision-7: the identity guarantee is unchanged, but the file moves to `<git-dir>/tandem-actor-id` (`.tandem/actor-id` outside Git) so it is never inside the synced board. Narrowly supersedes decision-8 for storage location, the one-time `migrate` command, and removal of `checkpoint`; its CLI, roles, Accord, and architecture decisions stand. Supersedes the source-branch checkpoint contracts from tasks 14, 37–40, 43, 44, and 46.

