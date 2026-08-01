---
id: task-139
type: task
title: "Specify canonical Epic, Task, and Subtask semantics"
priority: "critical"
references: ["decision-7", "decision-4"]
relatedFiles: ["AGENTS.md", "plan/spec.md", "plan/todo.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md", "tandem/plan/spec.md", "plan/delegated-task-tree-worker-spec.md"]
tags: ["protocol", "spec", "instructions", "hierarchy", "ids", "delegation"]
createdAt: "2026-07-15T19:44:23Z"
updatedAt: "2026-07-17T02:47:21Z"
parentId: "task-134"
accord:
  status: "accepted"
  assignee: "parent-orchestrator"
  claimedAt: "2026-07-17T02:46:31Z"
  deliveredAt: "2026-07-17T02:46:56Z"
  deliverables: ["Canonical Epic → global Task → parent-derived Subtask protocol and agent guidance", "Strict role/ID validation and reparenting requirements", "Task-only Worker A campaign semantics with pi-todos projection"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml -- --check — passed", "cargo test --manifest-path tandem/Cargo.toml — 146 passed", "git diff --check — passed", "Worker scoped contradiction and three-segment ID scans — passed"]
  constraints: ["Decision-7 fully supersedes decision-4 with no compatibility exception.", "Task-139 is a globally numbered Task of Epic task-134."]
  summary: "Accepted after parent inspection and independent 146-test validation; canonical documentation is consistent and non-visual."
  evidence: ["Commit f7b4be021beb16688ac6342fd7ab28508eab46ef", "Parent reviewed focused diff against b8f1f06 and post-merge test results"]
  filesChanged: ["AGENTS.md", "plan/spec.md", "plan/todo.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md", "tandem/plan/spec.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-17T02:47:04Z"
completedAt: "2026-07-17T02:47:21Z"
completion:
  summary: "Specified canonical Epic, Task, and Subtask roles, global Task versus parent-derived Subtask IDs, strict validation/reparenting rules, decision-4 supersession, and Task-only delegated campaigns."
  filesChanged: ["AGENTS.md", "plan/spec.md", "plan/todo.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md", "tandem/plan/spec.md"]
  validation: "Parent reviewed commit f7b4be0; cargo fmt passed; all 146 Rust tests passed post-merge; diff and contradiction/ID scans passed."
  reviewer: "parent-orchestrator"
---

## Description

This is a direct Task of Epic task-134. Correct canonical project direction before implementation using decision-7, which fully supersedes decision-4.

Acceptance criteria:
- Update AGENTS.md, protocol specification/todos, parent plans, and implementation specs to define the canonical Epic → Task → Subtask roles and nomenclature.
- Specify global `task-N` allocation for Epics and Tasks, including Tasks directly under Epics; reserve parent-derived `task-N-M` IDs for Subtasks under Tasks only.
- Remove statements that every task parent creates a Subtask, that direct Epic Tasks use hierarchical IDs, or that invalid forms receive compatibility treatment.
- Define `epic-task`, `subtask`, and generic `parent` classification from resolved documents and matching canonical IDs.
- Define strict validation: Epics cannot have parentId; Subtasks cannot have children; invalid role-changing reparenting is rejected; generic-parent Tasks may have Subtasks.
- Define Task-only delegation: Epics are not delegated, Subtasks are Worker A checklist items, and nested Worker B is deferred.
- Correct decomposition guidance: Epics contain independently managed globally numbered Tasks; Tasks contain parent-derived Subtasks when lifecycle-bearing checkpoints are needed.
- Do not claim CLI/TUI implementation is complete until later Tasks deliver it.
