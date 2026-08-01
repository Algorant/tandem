---
id: task-140
type: task
title: "Implement canonical hierarchy allocation, classification, and validation in the CLI"
priority: "critical"
blockers: ["task-139"]
references: ["decision-7"]
relatedFiles: ["tandem/src/main.rs", "tandem/plan/spec.md"]
tags: ["rust", "cli", "hierarchy", "validation", "json", "ids"]
createdAt: "2026-07-15T19:44:39Z"
updatedAt: "2026-07-22T02:53:16Z"
parentId: "task-134"
accord:
  status: "accepted"
  assignee: "shep:task-140"
  claimedAt: "2026-07-22T02:17:01Z"
  deliveredAt: "2026-07-22T02:52:25Z"
  deliverables: ["Shared HierarchyIndex over board and logs with Epic/Task/Subtask roles", "Strict global Task and parent-derived Subtask allocation", "Prospective mutation and descendant validation with canonical relationship output", "Cooperative lock coverage for graph-sensitive CLI reads and mutations", "Aggregated structural diagnostics and unsupported kind casing rejection", "Focused concurrency, allocation, graph, relationship, and update tests"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml -- --check — passed", "cargo check --manifest-path tandem/Cargo.toml — passed", "cargo test --manifest-path tandem/Cargo.toml — 148 passed", "git diff --check f7b4be0..58d937ac — passed", "Independent temporary-workspace hierarchy/concurrency/mutation smoke — passed", "Strict real-workspace list/show/log/decision reads after task-143-1 migration — passed", "All 14 migrated historical children verified as epic-task"]
  summary: "Accepted after parent diff review, targeted mutation/output/concurrency smokes, 148 passing Rust tests, successful fast-forward integration through 58d937a, and strict reads against the migrated real workspace."
  evidence: ["Commits 4baccdaa4d1ca1b76a2e9e399d368a7b171b9507 and 58d937ac4733903a5b264464575c2fa84ddb700e", "Branch shep/task-140-implement-canonical-hierarchy-allocation", "Worktree clean; only tandem/src/main.rs changed", "Independent audit subagent timed out without findings; parent performed direct diff and behavioral review"]
  filesChanged: ["tandem/src/main.rs"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-07-22T02:53:09Z"
completedAt: "2026-07-22T02:53:16Z"
completion:
  summary: "Integrated canonical decision-7 CLI hierarchy allocation, classification, validation, relationship output, graph locking, and aggregated diagnostics in commits 4baccda and 58d937a."
  filesChanged: ["tandem/src/main.rs"]
  validation: "Parent reviewed combined diff; cargo fmt/check passed; 148 Rust tests passed; temporary-workspace allocation/concurrency/mutation smoke passed; strict real-workspace reads passed after task-143-1 migration; /usr/bin/tandem unchanged."
---

## Description

This is a direct Task of Epic task-134. Implement decision-7 as the canonical Rust allocator, classifier, and mutation validator.

Acceptance criteria:
- Allocate global `task-N` IDs for root Tasks, Epics, generic-parent Tasks, and Tasks directly beneath Epics.
- Allocate parent-derived `task-N-M` IDs only for Subtasks directly beneath Task-role documents.
- Derive Epic, Task, and Subtask roles from kind, canonical ID form, and resolved parent chain across active board documents and completed logs.
- Add `epic-task` alongside `subtask` and generic `parent` relationship output.
- Human add/show/update output calls direct Epic children Tasks and reserves Subtask language for children of Tasks.
- Showing an Epic computes a Tasks collection; showing a Task computes Subtasks; non-task parents do not fabricate either.
- Reject parentId on Epics, nested Epics, children beneath Subtasks, ID/role mismatches, and reparenting that would cross canonical Task/Subtask nomenclature.
- Do not implement compatibility or migration shims for direct Epic children with Subtask-shaped IDs.
- Update list/search/show JSON, filters, warnings/errors, completion/log lookups, and focused CLI tests for allocation, Task-only delegation metadata, invalid depth, cycles, and reparenting.
