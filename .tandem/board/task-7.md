---
id: task-7
type: task
title: "Implement Rules and Decisions TUI views"
state: "in-progress"
priority: "medium"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/src/main.rs"]
tags: ["tui", "rules", "decisions", "mvp"]
createdAt: "2026-06-27T03:59:00Z"
updatedAt: "2026-06-27T16:36:40Z"
subtasks:
  - id: task-7-1
    title: "Rules list/add/edit/delete flow"
    completed: false
  - id: task-7-2
    title: "Decisions list/show/add flow"
    completed: false
  - id: task-7-3
    title: "Preserve protocol terminology and event names"
    completed: false
accord:
  status: "delivered"
  assignee: "herd:tui-rules-decisions"
  claimedAt: "2026-06-27T14:48:05Z"
  deliveredAt: "2026-06-27T16:36:40Z"
  deliverables: ["code:tandem-tui/src/tui.rs:Rules and Decisions views"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Stay aligned with existing CLI v0 behavior; protocol semantics should not change."]
  summary: "Revised task-7 implementation to keep Rules and Decisions TUI views in separate source modules while preserving behavior"
  evidence: ["Revision commit 3d5d575. Validation passed in worktree: cargo fmt --check, cargo test, cargo build, git diff --check HEAD~1..HEAD."]
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/rules.rs", "tandem-tui/src/tui/decisions.rs", "tandem-tui/src/tui/rules_decisions.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  updatedAt: "2026-06-27T16:36:40Z"
---

## Description

Add TUI surfaces for project rules and decision documents, matching the existing v0 CLI capabilities.
