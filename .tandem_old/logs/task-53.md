---
id: task-53
type: task
title: "Make tandem init title optional"
priority: "medium"
relatedFiles: ["tandem/src/main.rs", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md"]
tags: ["cli", "init", "ux"]
createdAt: "2026-06-29T00:03:17Z"
updatedAt: "2026-06-29T01:27:21Z"
subtasks:
  - id: task-53-1
    title: "Allow init command to run without --title"
    completed: false
  - id: task-53-2
    title: "Default omitted title to exact initialized directory basename"
    completed: false
  - id: task-53-3
    title: "Preserve explicit --title override behavior"
    completed: false
  - id: task-53-4
    title: "Add or update tests/validation for omitted and explicit title paths"
    completed: false
  - id: task-53-5
    title: "Update docs/spec/todo references if they currently imply title is required"
    completed: false
accord:
  status: "delivered"
  deliveredAt: "2026-06-29T00:06:15Z"
  validation:
    commands: ["cd tandem && cargo fmt && cargo test", "cd tandem && cargo build", "focused CLI smoke: target/debug/tandem init in a temp directory derives `Exact Project.Name`; `--title \"Explicit Demo\"` still overrides"]
  summary: "Implemented optional `tandem init` title: plain init derives the workspace title from the current directory basename with `Tandem Workspace` fallback, while explicit `--title` still overrides and validates non-empty input. Updated CLI help, missing-workspace hint, README/spec/todo docs, and added a focused unit test."
  evidence: ["cargo test: 60 passed", "cargo build: finished dev profile", "CLI smoke confirmed generated .tandem/tandem.md title for omitted and explicit title cases"]
  filesChanged: ["tandem/src/main.rs", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md"]
  updatedAt: "2026-06-29T00:06:15Z"
completedAt: "2026-06-29T01:27:21Z"
completion:
  summary: "Accepted as test-verifiable CLI init behavior. `tandem init` now succeeds without title, deriving exact directory basename with fallback and preserving explicit title override."
  filesChanged: ["tandem/src/main.rs", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md"]
  validation: "cargo fmt --check && cargo test passed; focused init smoke passed"
---

## Description

Plain `tandem init` should succeed without requiring a title. When `--title` is omitted, derive the workspace title from the basename of the directory being initialized and preserve that basename exactly. Keep `--title` as an explicit override. Avoid interactive prompts; init should remain script/agent friendly. Handle edge cases with a safe fallback such as `Tandem Workspace` when no meaningful directory basename exists.
