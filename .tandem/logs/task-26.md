---
id: task-26
type: task
title: "Papercuts from task-23/24 verification: disposition exit code, `$ ` whitespace, released state"
priority: "low"
effort: "small"
tags: ["protocol", "papercut", "accord"]
accord:
  status: "accepted"
  acceptance: ["`accord release --disposition bogus` exits 2 (usage) instead of 1, matching the protocol's exit-code contract.", "`$   echo x` classifies as a command with text `echo x`: strip the `$ ` prefix and then leading whitespace, or document that only one space is removed.", "Decide and document whether `accord release` should return `state` to `todo`; today a released Task keeps `state: in-progress` with `accordStatus: ready`, so a discarded attempt sits in the WIP column with no assignee. If intentional, say so in protocol/README.md; if not, reset state on release."]
  claimedAt: "2026-09-09T15:15:16Z"
  deliveredAt: "2026-09-09T15:15:42Z"
  validation: ["$ cargo test -p tandem", "Live: rerun the three commands above in a throwaway workspace and observe the documented behavior."]
  summary: "Resolved the verification papercuts: usage exit code, validation whitespace normalization, and released-task workflow state."
  evidence: ["`cd tandem && cargo test` passes all 262 unit tests plus 24 integration tests.", "Live release with `--disposition bogus` exits 2 and reports the usage error; `$   echo x` is returned by assignment JSON as a command with text `echo x`.", "Live valid release returns the Task to `state: todo` with `accordStatus: ready`, making it claimable again.", "Protocol documentation records the normalized validation command text and released-task state behavior."]
  filesChanged: ["protocol/README.md", "protocol/assignment.md", "tandem/src/app/assignment.rs", "tandem/src/app/error.rs", "tandem/src/main.rs", "tandem/src/protocol/accord.rs", "tandem/tests/accord_behavior.rs"]
  updatedAt: "2026-09-09T15:15:46Z"
createdAt: "2026-09-09T14:51:09Z"
updatedAt: "2026-09-09T15:15:46Z"
assignee: "Algorant"
archivedAt: "2026-09-09T15:15:46Z"
resolution:
  outcome: "completed"
---

## Description


Observed on the 0.12.4 dev build while verifying task-23 and task-24 in a fresh workspace. None blocks the release or the Pi adapter work. The third item matters most for the Pi `worker_discard` flow (~/.pi task-104): after discard the Task should look claimable on the Board, not in progress.

