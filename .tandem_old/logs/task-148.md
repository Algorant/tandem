---
id: task-148
type: task
title: "Establish the architecture campaign branch and governance baseline"
priority: "high"
parentId: "task-146"
references: ["task-145", "decision-7", "task-167", "decision-8"]
relatedFiles: ["plan/refactor_spec.md", "tandem/plan/modularization-research.md", "AGENTS.md", "plan/spec.md"]
tags: ["docs", "architecture", "refactor", "coordination"]
createdAt: "2026-07-22T20:40:11Z"
updatedAt: "2026-07-26T21:21:00Z"
blockers: ["task-167"]
accord:
  status: "accepted"
  assignee: "worker-task-148-b846032e"
  claimedAt: "2026-07-26T21:16:20Z"
  deliveredAt: "2026-07-26T21:20:47Z"
  deliverables: ["Commits dd362c7 and 1aeae17 integrated on refactor/protocol-architecture", "AGENTS.md campaign governance", "plan/refactor_campaign_baseline.md controlled base and synchronization rules"]
  validation:
    commands: ["git show --check 1aeae17", "Worker worktree clean", "Integration branch merge-base with main equals 355e76c9a51a90b5b41b383bbc4c4efe5ffa74e5", "No Rust or Cargo files changed", "origin/HEAD remains origin/main"]
  summary: "Established and integrated the architecture campaign governance baseline on refactor/protocol-architecture, with accepted decision-8 as architecture authority and decision-7 retained for hierarchy."
  evidence: ["decision-8 accepted and referenced by task-146/task-148", "git diff 355e76c..1aeae17 reviewed", "git show --check passed", "refactor/protocol-architecture now points to 1aeae17", "worker and integration worktrees clean"]
  filesChanged: ["AGENTS.md", "plan/refactor_campaign_baseline.md"]
  reviewer: "parent-orchestrator"
  note: "Reviewed both commits and corrected the architecture-decision reference through retained Worker rework. Integrated exact reviewed commits into the dedicated branch and independently verified base, clean status, default branch, and absence of Rust/Cargo changes."
  updatedAt: "2026-07-26T21:20:51Z"
assignee: "worker-task-148-b846032e"
completedAt: "2026-07-26T21:21:00Z"
completion:
  summary: "Created and integrated the controlled refactor/protocol-architecture baseline at main commit 355e76c; added campaign governance in AGENTS.md and plan/refactor_campaign_baseline.md; recorded accepted architecture decision-8 and retained decision-7 as hierarchy authority; verified no Rust/Cargo changes or push."
  filesChanged: ["AGENTS.md", "plan/refactor_campaign_baseline.md"]
  validation: "Reviewed commits dd362c7 and 1aeae17; git show --check passed; integration merge-base is 355e76c9a51a90b5b41b383bbc4c4efe5ffa74e5; worker and integration statuses clean; origin/HEAD remains origin/main; no Rust or Cargo files changed."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Establish the controlled starting point for Epic task-146 after the broad architecture decision is accepted and before any Rust implementation moves.

## Scope

- Confirm `plan/refactor_spec.md`, Epic task-146, its direct Tasks, and the accepted architecture decision agree.
- Create `refactor/protocol-architecture` from the explicitly approved clean `main` commit.
- Record the integration base and branch synchronization rules.
- Add only the initial project/agent guidance needed to enforce the Rust CLI/TUI freeze on `main`, Task-only delegation, review-before-integration, module checkpoints, and human TUI validation requirements.
- Verify that unrelated documentation and extension work may continue without changing frozen Rust architecture.

## Acceptance criteria

- The accepted architecture decision exists and is referenced by the campaign records before this Task starts.
- The integration branch has an unambiguous base and clean status.
- Initial guidance names `protocol`, `project`, `app`, `cli`, and `tui` boundaries without pretending they already exist.
- No production Rust refactor, protocol behavior change, dependency change, release, or push occurs.
- Validation records branch/base evidence and confirms `main` remains the release/default branch.

Creating this Task does not authorize starting it.
