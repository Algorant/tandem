---
id: task-23
type: task
title: "Expose attempt, rework, and discard counts on the Accord"
state: "in-progress"
priority: "medium"
effort: "medium"
relatedFiles: ["protocol/README.md", "protocol/assignment.md"]
tags: ["protocol", "accord", "delegation"]
accord:
  status: "claimed"
  acceptance: ["`accord release` accepts an optional `--disposition discarded|reassign` (default reassign); the value is written to the accord.released event data and the release note remains required.", "`show --json` and `assignment --json` for an active or archived Task include derived `attemptCount` (accord.claimed events), `reworkCount` (accord.rework events), and `discardedCount` (accord.released with disposition discarded); counts are computed from the Task's events, not stored as editable fields.", "The TUI task detail shows the three counts when any is non-zero.", "protocol/README.md documents the disposition and the derived counts, and states that adapters should record post-delivery corrections as `accord rework` rather than out-of-band messages.", "An explicit adapter handoff is recorded for pi-tandem/pi-agency to consume the native counts (see ~/.pi task-107); this Task does not modify extensions/pi-tandem."]
  claimedAt: "2026-09-09T14:30:35Z"
  validation: ["$ cargo test -p tandem", "Live: claim, deliver, rework, deliver, release --disposition discarded, claim, deliver, complete a throwaway Task; `tandem show --json` reports attemptCount 2, reworkCount 1, discardedCount 1 on the archived record."]
  constraints: ["No new Accord status; discarded is a release disposition, not a lifecycle state.", "Counts are per Task across all attempts; do not add per-attempt sub-records in this Task."]
  updatedAt: "2026-09-09T14:30:35Z"
createdAt: "2026-09-09T14:22:46Z"
updatedAt: "2026-09-09T14:30:35Z"
assignee: "Algorant"
---

## Description


## Source

Pi delegation review of the dotfiles epic: ~/.dotfiles/workflow_issues/worker-dispatch-review-recommendations.md (commit 4392d5e6). Inspection of that workspace's `.tandem/events` showed one claim, one deliver, one complete per Task despite four-plus correction cycles, because the Pi adapter routed corrections as messages instead of `accord rework`. The adapter side is being fixed in ~/.pi (task-102 epic, task-107). Tandem's part is to make the counts visible so the owner can see churn on a Task without reconstructing it from chat, and to give a discarded attempt a durable disposition distinct from ordinary reassignment.

Events already carry everything needed; this is a projection plus one release flag.

## Not in scope

Active/waiting time. That is runtime-observed and belongs to the adapter or Herdr; Tandem should not guess it.

