---
title: Concepts
description: Tasks, Accords, Decisions, Rules, and board sync.
---

## Tasks and Accords

A Task is a unit of work; most Tasks need acceptance criteria, while a title-only papercut is the exception. An Accord records who claimed it and what they delivered. `accord claim` starts work, `accord deliver` records a summary and evidence, and `complete` normally accepts a delivered Task and moves it to Logs. If the result needs a human check, `review` sends one acceptance criterion to `validation`; `accord rework` returns it to work. Review status is not a stored field.

An Epic groups Tasks; a Task may have leaf Subtasks. Decisions record durable choices and begin as `proposed`. Rules in `.tandem/rules/` give people and agents project guidance.

[![Tasks and review move through the shared work lifecycle.](/images/work-and-review.png)](/images/work-and-review.png)

## Board storage and sync

In a Git repository, `.tandem/` lives in the main worktree, is ignored by source Git, and syncs independently through the repository's `tandem` branch. Linked worktrees use the same board. Board changes are not source commits. Tandem manages the branch; do not manually commit or merge it.

A board outside Git is local to that directory. For multiple machines, install a compatible Tandem version everywhere, configure the same remote, and sync the board through `tandem`. Offline changes remain saved locally and are reported as **pending** until the remote accepts them.

[![Source Git and the Tandem-managed board use separate histories.](/images/source-and-tandem-sync.png)](/images/source-and-tandem-sync.png)

If two machines change the same record differently, the remote version remains on the shared board while Tandem preserves both versions locally and reports a conflict. Inspect it with `tandem sync status`, then resolve the reported record with `tandem sync resolve <id> --keep local|remote|edited`. Tandem may also hold back a local edit that would make the shared board invalid; fix the reported record and sync again. A conflict or held edit does not discard the saved local work.

[![A board change waiting to sync is reported as pending.](/images/sync-status-pending.png)](/images/sync-status-pending.png)

[![A same-record conflict is preserved for resolution.](/images/sync-status-conflict.png)](/images/sync-status-conflict.png)

[![An invalid local record is held out of the shared board.](/images/sync-status-held.png)](/images/sync-status-held.png)

[![The TUI identifies a record with a sync conflict.](/images/tui-sync-conflict.png)](/images/tui-sync-conflict.png)

See the [Quickstart](/quick-start/) for a complete Task, and the [CLI reference](/cli/) for sync commands.
