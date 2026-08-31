---
id: task-106
type: task
title: "Specify first-class subtask protocol convention"
priority: "medium"
parentId: "task-101"
relatedFiles: ["protocol/plan/spec.md", "protocol/README.md", "plan/spec.md"]
tags: ["protocol", "subtasks", "relationships"]
createdAt: "2026-07-05T16:23:00Z"
updatedAt: "2026-07-10T17:49:05Z"
accord:
  status: "accepted"
  assignee: "shep-task-106"
  claimedAt: "2026-07-10T17:26:18Z"
  deliveredAt: "2026-07-10T17:48:34Z"
  deliverables: ["Focused commit 5b49bcac1f17b3dec64f03e39273291233745fca on branch shep/task-106-specify-first-class-subtask-protocol-con", "Updated plan/spec.md, protocol/README.md, and protocol/plan/spec.md with consistent first-class subtask semantics", "Documented compatibility boundaries for existing inline checklist subtasks and flat default ID allocation"]
  validation:
    commands: ["Parent inspected the complete commit diff and found it scoped to the three intended files", "cargo test --manifest-path tandem/Cargo.toml — 124 passed, 0 failed (worker and parent independently)", "git diff HEAD^ HEAD --check — passed", "Markdown fence-balance check — passed", "Targeted semantic/stale-claim searches — expected new convention present; stale contradictory phrases absent", "merge-tree against current main — no conflict markers", "Worker git status --short — clean; no unexpected files"]
  summary: "PASS. Parent reviewed the full scoped documentation diff, independently reran automated and semantic validation, and fast-forwarded commit 5b49bca to main. The work satisfies the task's objective non-visual protocol acceptance criteria."
  evidence: ["Branch: shep/task-106-specify-first-class-subtask-protocol-con", "Worktree: /home/ivan/.pi/agent/worktrees/tandem/task-106-specify-first-class-subtask-protocol-con", "Commit: 5b49bcac1f17b3dec64f03e39273291233745fca", "Worker reported downstream CLI/TUI/agent guidance may still need alignment, which is already tracked by sibling tasks 103-105"]
  filesChanged: ["plan/spec.md", "protocol/README.md", "protocol/plan/spec.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-10T17:48:59Z"
completedAt: "2026-07-10T17:49:05Z"
completion:
  summary: "Defined and adopted the protocol convention for first-class subtasks: normal task documents linked by parentId, optional hierarchical IDs, legacy inline checklist compatibility, and hierarchy distinct from epic classification."
  filesChanged: ["plan/spec.md", "protocol/README.md", "protocol/plan/spec.md"]
  validation: "Parent reviewed the full diff and integrated commit 5b49bcac1f17b3dec64f03e39273291233745fca by fast-forward. `cargo test --manifest-path tandem/Cargo.toml` passed 124 tests; diff checks, markdown fence balance, semantic consistency searches, clean worktree, and conflict checks passed."
  reviewer: "parent-orchestrator"
---

## Description

Update Tandem protocol/spec language for first-class subtasks.

Scope:
- Define a subtask as a normal `type: task` document linked to another task with `parentId`.
- Clarify hierarchical IDs like `task-100-1` are recommended/allowed when useful, not required.
- Clarify inline `subtasks:` checklist items are legacy/deprecated for new work.
- Distinguish epics (`kind: epic`) from general parent/child subtask hierarchy.
- Avoid adding a new document type or new relationship field unless the spec review finds a concrete need.
