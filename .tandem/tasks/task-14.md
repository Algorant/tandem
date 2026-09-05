---
id: task-14
type: task
title: "Checkpoint tracked Tandem state natively at work boundaries"
state: todo
priority: "medium"
relatedFiles: ["tandem/src/project/write.rs", "tandem/src/app/accord.rs", "tandem/src/app/tasks.rs", "protocol/plan/spec.md"]
tags: ["protocol", "git"]
accord:
  status: "ready"
  acceptance: ["One documented native boundary policy checkpoints root work start, delivery, block/pause and finish while intermediate progress/metadata edits persist without per-edit Git commits; no new daemon required.", "Real-Git tests prove preservation of unrelated staged/unstaged/untracked state, bounded checkpoint history across repeated lifecycle actions and safe concurrent boundary calls.", "Pushed or unproven ordinary commits are not amended; native record writes and Git-checkpoint failures have distinct truthful results so retries cannot repeat successful mutations.", "CLI and TUI invoke the same native behavior, and a Pi consumer handoff defines how to remove the old adapter-owned housekeeping without two paths running together."]
  validation: ["Use temporary real repositories to assert file/index/HEAD states, not only mocked git argv; run repository-prescribed Rust tests."]
  constraints: ["No Pi adapter mutation in this core Task; provide explicit follow-up handoff.", "Keep all Tandem content tracked; never commit checkout-local actor identity, caches or credentials.", "No force/hook bypass, fallback Git path, attempt archive or one-commit-per-progress-item policy."]
  updatedAt: "2026-09-05T21:03:46Z"
createdAt: "2026-09-05T21:03:46Z"
updatedAt: "2026-09-05T21:03:46Z"
---

## Description

Implementation proposal from Pi task-59. All .tandem state stays tracked. User selected immediate durable record writes but automatic batched Git checkpoints at work boundaries (start/claim, delivery, pause/block, completion), not every progress edit. The same behavior must work from native CLI/TUI and Pi, without a background coordinator service or duplicate Pi housekeeping path. Root/per-milestone updates must not turn every local step into a checkpoint boundary.

Use current accepted safety behavior in /home/ivan/.pi/agent/extensions/pi-agency/tandem-housekeeping.ts and Pi task-47 as evidence, not mandatory architecture. Stage/commit only the owning .tandem path, preserve unrelated staged/unstaged/untracked bytes and index state, never rewrite pushed or unproven commits, and serialize native Git operations per repository. Native writes stay truthful if checkpointing fails: report the failure separately rather than making an already-successful lifecycle action look safe to retry. Batch intermediate changes at the next actual work boundary; do not create an audit/attempt history system.

Publish a consumer handoff specifying boundary policy, commands/results and availability. Pi removes its existing housekeeping implementation only after the native path is usable; final cutover must leave one path. Coordinate shared app mutation entry-point ownership with other Tandem work before dispatch.
