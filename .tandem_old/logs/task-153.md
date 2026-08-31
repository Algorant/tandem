---
id: task-153
type: task
title: "Extract canonical protocol IDs and hierarchy authority"
priority: "high"
parentId: "task-146"
blockers: ["task-152"]
references: ["decision-7", "task-134"]
relatedFiles: ["plan/refactor_spec.md", "protocol/plan/spec.md", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/protocol/ids.rs", "tandem/src/protocol/hierarchy.rs"]
tags: ["protocol", "rust", "hierarchy", "refactor"]
createdAt: "2026-07-22T20:41:19Z"
updatedAt: "2026-07-26T22:13:40Z"
accord:
  status: "accepted"
  assignee: "worker-task-153-a3c85044"
  claimedAt: "2026-07-26T22:06:03Z"
  deliveredAt: "2026-07-26T22:13:26Z"
  deliverables: ["Canonical global Task and parent-derived Subtask ID queries", "Canonical role and relationship derivation from resolved documents", "Structural hierarchy validation and metadata checks", "Board+Logs allocation query fed by the existing locked snapshot adapter", "CLI/TUI/root callers migrated off duplicate root authority", "Focused protocol hierarchy/ID tests"]
  validation:
    commands: ["cargo fmt --check", "177 unit + 4 executable tests passed", "2 concurrency allocation tests passed", "focused hierarchy and executable hierarchy tests passed", "strict Clippy passed", "duplicate-authority search clean", "no new hierarchy/ID suppressions"]
  summary: "Extracted canonical protocol ID grammar/allocation queries and resolved hierarchy authority into protocol/ids.rs and protocol/hierarchy.rs."
  evidence: ["commit 26a2a97 fast-forward integrated", "177 unit + 4 executable tests passed", "concurrency and focused hierarchy tests passed", "strict Clippy/format passed", "single TaskRole/ParentRelationship/HierarchyIndex authority found", "protocol hierarchy performs no file discovery/read/write/locking"]
  filesChanged: ["tandem/src/protocol/ids.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/protocol/mod.rs", "tandem/src/main.rs", "tandem/src/tui/mod.rs"]
  reviewer: "parent-orchestrator"
  note: "Reviewed the full extraction and independently reran all required gates. Decision-7 roles remain resolved-document-derived, direct Epic children remain global Tasks, Task children remain parent-derived Subtasks, and allocation still consumes the locked Board+Logs snapshot. The known source-bearing StoredDocument/CLI-error coupling is an explicit staged seam for task-154/task-155, not a duplicate hierarchy implementation or concrete filesystem operation inside protocol."
  updatedAt: "2026-07-26T22:13:32Z"
assignee: "worker-task-153-a3c85044"
completedAt: "2026-07-26T22:13:40Z"
completion:
  summary: "Moved canonical task ID grammar/allocation and all resolved Epic/Task/Subtask hierarchy semantics into protocol/ids.rs and protocol/hierarchy.rs; migrated CLI/TUI callers and removed duplicate root inference while retaining locked Board+Logs snapshot inputs."
  filesChanged: ["tandem/src/protocol/ids.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/protocol/mod.rs", "tandem/src/main.rs", "tandem/src/tui/mod.rs"]
  validation: "Parent reviewed commit 26a2a97 and passed formatting, 177 unit tests, 4 real-command tests, focused hierarchy tests, both concurrency allocation tests, strict Clippy, and duplicate-authority/dependency audits. The temporary source-bearing StoredDocument diagnostic seam is explicitly deferred to the following diagnostic/project extraction checkpoints."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Move all ID and hierarchy semantics into one executable-protocol authority while preserving decision-7 behavior exactly.

## Scope

- Establish cohesive `protocol/ids.rs` and `protocol/hierarchy.rs` modules.
- Keep Epic/Task/Subtask role derivation, parent relationships, structural graph validation, role-specific ID forms, Board-and-Logs allocation without reuse, and hierarchy diagnostics together behind narrow APIs.
- Preserve the rule that roles derive from resolved documents rather than ID shape.
- Preserve strict rejection of parented Epics, children beneath Subtasks, role/ID mismatches, invalid reparenting, unresolved hierarchy references, and concurrent duplicate allocation.
- Switch every CLI/TUI/project-facing caller to the canonical implementation and remove duplicate inference.

## Acceptance criteria

- Searches and review find one hierarchy/ID implementation and no compatibility shim for decision-4 shapes.
- Direct Epic children remain global-ID Tasks; direct Task children remain parent-derived leaf Subtasks.
- Allocation scans active Board documents and completed Logs under the existing lock/concurrency contract.
- Existing hierarchy and concurrency tests move with code; executable CLI and focused TUI hierarchy tests remain green.
- Formatting, full tests, real-command tests, strict Clippy, and dependency/visibility review pass.
- Temporary lint expectations assigned to IDs/hierarchy are removed.
- No project-I/O extraction, output/UI redesign, release, or push occurs.

Creating this Task does not authorize starting it.
