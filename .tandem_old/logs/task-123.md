---
id: task-123
type: task
title: "Migrate repository pi-tandem adapter to first-class subtasks"
priority: "medium"
parentId: "task-105"
references: ["task-103", "task-106"]
relatedFiles: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md"]
tags: ["pi-tandem", "subtasks", "cli", "smoke"]
createdAt: "2026-07-10T19:32:10Z"
updatedAt: "2026-07-12T03:41:42Z"
accord:
  status: "accepted"
  assignee: "shep-task-123"
  claimedAt: "2026-07-10T19:32:43Z"
  deliveredAt: "2026-07-12T03:41:24Z"
  deliverables: ["Focused amended commit cf5c99397d38bb1cb842dcf2cb6699ad3bdb49de on shep/task-123-migrate-repository-pi-tandem-adapter-to", "Updated repository pi-tandem adapter schema/builders/guidance for tracked child tasks", "Expanded smoke coverage for parentId, parentRelationship, computed subtask summaries, parent filters, update reparenting, and deprecated inline authoring rejection", "Updated public README search mapping to include --parent"]
  validation:
    commands: ["Parent inspected the full adapter/docs/test diff and the amended documentation delta", "bun --check for adapter and three TypeScript smokes — passed", "bun extensions/pi-tandem/tests/smoke.ts — passed", "bun extensions/pi-tandem/tests/pi-runtime-smoke.ts — passed", "bun extensions/pi-tandem/tests/relationship-smoke.ts — passed", "cargo test --manifest-path tandem/Cargo.toml — 127 passed, 0 failed", "git diff HEAD^ HEAD --check — passed", "merge-tree against current main — no conflict markers", "Worker git status --short — clean; no unexpected files"]
  summary: "PASS. Parent reviewed the full repository adapter migration and amended README fix, independently reran all Bun adapter smokes/checks, confirmed 127 Rust tests and a clean conflict-free branch, and fast-forwarded cf5c993 to main."
  evidence: ["Branch: shep/task-123-migrate-repository-pi-tandem-adapter-to", "Worktree: /home/ivan/.pi/agent/worktrees/tandem/task-123-migrate-repository-pi-tandem-adapter-to", "Commit: cf5c99397d38bb1cb842dcf2cb6699ad3bdb49de", "Repository-read subsection correctly skipped in isolated worktree without .tandem; temporary-workspace mutation and Pi runtime coverage passed"]
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/plan/todo.md", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/tests/pi-runtime-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-12T03:41:35Z"
completedAt: "2026-07-12T03:41:42Z"
completion:
  summary: "Migrated repository-local pi-tandem to first-class parent-linked subtasks, removed deprecated inline authoring from its public schema/forwarding, added parent filters and update support, and aligned guidance plus comprehensive smokes with CLI-owned relationship output."
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/plan/todo.md", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/tests/pi-runtime-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.md"]
  validation: "Parent reviewed and integrated commit cf5c99397d38bb1cb842dcf2cb6699ad3bdb49de. Bun syntax checks and all three adapter smokes passed; 127 Rust tests passed; diff checks, clean status, documentation consistency, and merge checks passed."
  reviewer: "parent-orchestrator"
---

## Description

Update the repository-local `extensions/pi-tandem` adapter and tests for the accepted first-class subtask CLI behavior.

Scope and acceptance criteria:
- Stop forwarding deprecated inline `--subtask` authoring to Tandem; use `parent`/`parentId` child tasks for independently tracked work.
- Keep pi-tandem thin over the CLI and do not duplicate relationship classification or protocol mutation logic in TypeScript.
- Ensure task show/list/search responses naturally expose the CLI's `parentId`, `parentRelationship`, and computed subtask summaries where returned.
- Update relationship smoke coverage so it passes against the current source-built Tandem CLI and verifies parent-linked child behavior.
- Update repository-local pi-tandem guidance/spec text to describe inline checklist subtasks as legacy/deprecated rather than the future delegation unit.
- Run Bun tests/smokes and relevant Rust-backed integration validation.
