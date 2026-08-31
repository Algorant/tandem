---
id: task-101
type: task
kind: "epic"
title: "Define and adopt first-class subtask hierarchy"
priority: "medium"
relatedFiles: ["protocol/plan/spec.md", "tandem/plan/spec.md", "tandem/src", "extensions/pi-tandem", "docs", "site/src/content/docs"]
tags: ["protocol", "subtasks", "relationships"]
createdAt: "2026-07-05T16:22:12Z"
updatedAt: "2026-07-14T11:25:19Z"
accord:
  status: "accepted"
  assignee: "parent-orchestrator"
  claimedAt: "2026-07-10T17:26:00Z"
  deliveredAt: "2026-07-14T04:56:57Z"
  deliverables: ["Protocol and decision alignment in tasks 125/decision-4", "Hierarchical CLI allocation and compatibility in task-126", "Repository pi-tandem alignment in task-127", "Human-approved Epic Board hierarchy UX in task-129", "Public documentation in task-130"]
  validation:
    commands: ["Task-126: 129 Rust tests plus CLI and Bun relationship smokes passed", "Task-127: Bun adapter/runtime/relationship smokes and 129 Rust tests passed", "Task-129: 137 Rust tests, fmt/build/clippy, and human visual approval passed", "Task-130: 15-page Bun/Astro docs build and 602 internal link checks passed", "All integrated Git work through task-129 was pushed; task-130 is integrated locally on main"]
  constraints: ["Task-128 was intentionally closed unimplemented because canonical personal dotfiles are outside this Tandem-repository epic"]
  summary: "PASS. The epic's corrected decision-4 scope is fully implemented and documented in the Tandem repository. Objective protocol/CLI/adapter/docs validation passed, the Epic Board received explicit human visual approval, all intended child tasks are resolved, and the rejected dotfiles-only task was intentionally closed unimplemented."
  evidence: ["decision-4 accepted", "Tasks 102-106 and 123-130 resolved in logs", "No active Shep workers remain"]
  filesChanged: ["AGENTS.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/review.rs", "tandem/README.md", "tandem/plan/spec.md", "extensions/pi-tandem", "README.md", "docs"]
  reviewer: "pi"
  updatedAt: "2026-07-14T04:57:06Z"
references: ["decision-4"]
completedAt: "2026-07-14T11:25:19Z"
completion:
  summary: "Completed the first-class subtask hierarchy epic under accepted decision-4: new task children use CLI-allocated parent-derived and nested IDs, parentId remains canonical, IDs are immutable, existing flat children remain valid, inline checklist subtasks are legacy, repository pi-tandem stays thin, Epic Board hierarchy UX is human-approved, and public docs are current."
  filesChanged: ["AGENTS.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/review.rs", "tandem/README.md", "tandem/plan/spec.md", "extensions/pi-tandem", "README.md", "docs"]
  validation: "PASS. Child tasks 102-106 and 123-130 are resolved. Rust suites reached 137 passing tests; CLI and Bun adapter/runtime/relationship smokes passed; Epic Board received Algorant's visual approval; public docs built 15 pages and passed 602 internal links. Task-128 was intentionally closed unimplemented because personal canonical dotfiles are outside this repository epic."
  reviewer: "pi"
---

## Description

Epic for defining and adopting Tandem's first-class subtask model.

Direction:
- Treat subtasks going forward as normal `type: task` documents linked with `parentId`.
- Hierarchical IDs such as `task-100-1` are recommended only when useful, not required.
- Do not auto-generate hierarchical subtask IDs in v0.
- Deprecate inline `subtasks:` checklist creation for new work; no need to preserve it as the future authoring path.
- Delegation, validation, CLI, TUI, docs, and Pi/Shep surfaces should understand subtask shape and terminology.
- No new completion warn/block policy is required; validation/review workflow already handles judgment.
