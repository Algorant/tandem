---
id: task-26
type: task
title: "Papercuts from task-23/24 verification: disposition exit code, `$ ` whitespace, released state"
state: todo
priority: "low"
effort: "small"
tags: ["protocol", "papercut", "accord"]
accord:
  status: "ready"
  acceptance: ["`accord release --disposition bogus` exits 2 (usage) instead of 1, matching the protocol's exit-code contract.", "`$   echo x` classifies as a command with text `echo x`: strip the `$ ` prefix and then leading whitespace, or document that only one space is removed.", "Decide and document whether `accord release` should return `state` to `todo`; today a released Task keeps `state: in-progress` with `accordStatus: ready`, so a discarded attempt sits in the WIP column with no assignee. If intentional, say so in protocol/README.md; if not, reset state on release."]
  validation: ["$ cargo test -p tandem", "Live: rerun the three commands above in a throwaway workspace and observe the documented behavior."]
  updatedAt: "2026-09-09T14:51:09Z"
createdAt: "2026-09-09T14:51:09Z"
updatedAt: "2026-09-09T14:51:09Z"
---

## Description


Observed on the 0.12.4 dev build while verifying task-23 and task-24 in a fresh workspace. None blocks the release or the Pi adapter work. The third item matters most for the Pi `worker_discard` flow (~/.pi task-104): after discard the Task should look claimable on the Board, not in progress.

