---
id: task-119
type: task
title: "Research storage options for Tandem event logs outside the project repository"
priority: "medium"
relatedFiles: ["protocol/plan/spec.md", "tandem/src"]
tags: ["protocol", "events", "research", "git"]
createdAt: "2026-07-10T12:42:01Z"
updatedAt: "2026-07-12T13:56:02Z"
accord:
  status: "accepted"
  assignee: "shep-task-119"
  claimedAt: "2026-07-12T13:48:01Z"
  deliveredAt: "2026-07-12T13:55:36Z"
  deliverables: ["Focused research note at protocol/plan/event-storage-options.md", "Commit a95dae2208ae1fbc0b8e84d86b9f943ea4d2313b on shep/task-119-research-storage-options-for-tandem-even", "Options comparison, recommended default, migration implications, rollout sequence, and open questions"]
  validation:
    commands: ["Parent read the complete research note and confirmed required scope coverage", "Required terminology/options/evaluation/recommendation/migration/open-question/rollout section assertions — passed", "Markdown fence balance — passed", "git diff HEAD^ HEAD --check — passed", "merge-tree against current main — no conflict markers", "Worker git status --short — clean; no unexpected files"]
  summary: "PASS. Parent reviewed the complete research note, confirmed all requested options and evaluation dimensions are covered, verified it remains explicitly non-decisional, and fast-forwarded a95dae2 to main."
  evidence: ["Branch: shep/task-119-research-storage-options-for-tandem-even", "Worktree: /home/ivan/.pi/agent/worktrees/tandem/task-119-research-storage-options-for-tandem-even", "Commit: a95dae2208ae1fbc0b8e84d86b9f943ea4d2313b", "Document explicitly marks itself as research with no protocol decision"]
  filesChanged: ["protocol/plan/event-storage-options.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-12T13:55:52Z"
completedAt: "2026-07-12T13:56:02Z"
completion:
  summary: "Completed a concise event-log storage options analysis recommending external user-state storage for review, with tracked/ignored/Git/checkpoint/backend alternatives, migration implications, rollout stages, terminology, and open questions; no protocol decision was encoded."
  filesChanged: ["protocol/plan/event-storage-options.md"]
  validation: "Parent reviewed the complete research note and integrated a95dae2208ae1fbc0b8e84d86b9f943ea4d2313b. Required-section and terminology assertions, Markdown fence balance, diff checks, clean status, and merge checks passed."
  reviewer: "parent-orchestrator"
---

## Description

Research how Tandem can separate or clearly disambiguate append-only event logs from the user-authored project repository so routine event activity does not require or pollute normal Git commits.

Explore options both within and outside Git, including:
- keeping workspace-local events under `.tandem/` but ignored or otherwise separated from tracked project data;
- storing events in an external user/state directory keyed to the workspace;
- using a separate Git branch, worktree, repository, or other Git-native mechanism;
- retaining only compact checkpoints or selected audit records in the project repository;
- supporting configurable storage backends or modes.

Evaluate each option for local-first behavior, portability, multi-user/agent collaboration, audit durability, repository identity and relocation, merge/concurrency behavior, backup and recovery, privacy, discoverability, performance, backwards compatibility, and implementation complexity. Clarify terminology and UI/CLI presentation so users can distinguish workflow history, completed-work logs, and low-level event/audit logs.

Deliverable: a concise options analysis with a recommended default, viable alternatives, migration implications, and open questions. This is a research task; do not encode a protocol decision until reviewed.
