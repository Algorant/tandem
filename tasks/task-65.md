---
id: task-65
uid: dfcdecf2-b07b-4ac2-8375-3935f8f5d97b
type: task
kind: "papercut"
title: "tandem migrate: no supported way to drop an unsynced local record when the shared board is already upgraded"
state: todo
priority: "low"
tags: ["migrate", "sync"]
accord:
  status: "ready"
  acceptance: ["A user can discard or set aside unsynced local board records through a supported Tandem command or flag (e.g. `tandem migrate --discard-local` or a sync subcommand). The command keeps a recoverable copy, and the error message names it."]
  updatedAt: "2026-10-06T19:04:48Z"
createdAt: "2026-10-06T19:04:48Z"
updatedAt: "2026-10-06T19:04:48Z"
---

## Description

Seen 2026-10-06 on cart-lab's ~/.pi board, which was still on 0.4.0 after the shared board had moved to 0.5.0.

- **The refusal:** `tandem migrate` refused with "this checkout has changes that were never synced … Sync them with the previous Tandem release first, or move them aside". The local change was a single Task created on Oct 2 that duplicated a record already on the shared board (task-329, different uid).
- **"Sync with the previous release" was impossible:** 0.15.0 refused to sync because the combined board would be invalid (new kinds).
- **"Move aside" isn't something a user can do:**
  - Deleting the files does nothing: `restore_missing` brings them back from `refs/tandem/pending`.
  - The only fix was manual: delete `refs/tandem/pending` and remove the two files (task + event) together, then migrate.
  - Agents are forbidden to touch `.tandem` or its refs, so this needed Algorant's explicit authorization.
