---
id: task-160
type: task
title: "Extract TUI reload, input, and Validation adapter boundaries"
priority: "high"
parentId: "task-146"
blockers: ["task-159"]
references: ["task-145"]
relatedFiles: ["plan/refactor_spec.md", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/input.rs", "tandem/src/tui/validation.rs"]
tags: ["tui", "rust", "architecture", "refactor"]
createdAt: "2026-07-22T20:42:43Z"
updatedAt: "2026-07-28T22:37:12Z"
accord:
  status: "accepted"
  assignee: "worker-task-160-55fc722e"
  claimedAt: "2026-07-28T21:31:58Z"
  deliveredAt: "2026-07-28T21:37:50Z"
  deliverables: ["Commit fd6a1138c4cce51c6669357df7c31b49d9a0f7e7", "tandem/src/tui/input.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/validation.rs", "tandem/src/tui/mod.rs"]
  validation:
    commands: ["Focused TUI tests: 82 passed", "Independent parent rerun: cargo fmt --check passed", "Independent parent rerun: cargo clippy --all-targets -- -D warnings passed", "Independent parent rerun: cargo test passed (206 unit + 6 real-command tests)", "Worker PTY smoke started TUI and exited cleanly via q", "git diff --check passed"]
  constraints: ["Human just dev visual/interactive validation remains required before acceptance or integration"]
  summary: "Accepted after code review, independent formatting/Clippy/full-test validation, clean integration, and successful human just dev visual/interactive validation."
  evidence: ["Worker checkout clean", "Parent diff inspection confirmed move-focused extraction into three cohesive modules", "Input adapter contains no direct filesystem or app mutation calls"]
  filesChanged: ["tandem/src/tui/input.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/validation.rs", "tandem/src/tui/mod.rs"]
  reviewer: "orchestrator"
  updatedAt: "2026-07-28T22:37:08Z"
assignee: "worker-task-160-55fc722e"
completedAt: "2026-07-28T22:37:12Z"
completion:
  summary: "Extracted TUI reload, input translation, and Validation adapter boundaries; integrated commit 6b51a0c after automated and human validation."
  filesChanged: ["tandem/src/tui/input.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/validation.rs", "tandem/src/tui/mod.rs"]
  validation: "Human just dev validation passed; cargo fmt --check, strict Clippy, 206 unit tests, 6 real-command tests, PTY smoke, and git diff checks passed."
  reviewer: "orchestrator"
---

## Description

## Objective

Separate TUI aggregate/reload behavior, event translation, and Validation UI adapters while preserving all visible and durable behavior.

## Scope

- Extract reload, fingerprints, external-change handling, warning aggregation, and selection restoration into a cohesive TUI boundary.
- Separate keyboard/mouse translation from durable application operations; introduce one small shared UI action value only where existing key/hit paths demonstrably converge.
- Extract Validation prompts, accept/rework/apply UI state, and calls into shared app operations.
- Preserve modal ownership, focus, scrolling, filtering, selection, quit behavior, hit-map geometry, status messages, and non-panicking tolerant reload diagnostics.
- Keep Ratatui-pragmatic mutable rendering; do not introduce Elm/Redux, component traits, an event bus, or async runtime.

## Acceptance criteria

- Input translation performs no direct file writes; rendering performs no app mutation.
- Keyboard and mouse paths remain equivalent where they represent the same action.
- External-change reload and selection restoration remain stable.
- Existing focused input/reload/Validation/render tests, full tests, real-command tests, PTY checks, formatting, and strict Clippy pass.
- Genuine human `just dev` validation approves visible behavior.
- Temporary lint expectations assigned to these TUI seams are removed.
- No Board decomposition, protocol change, release, or push occurs.

Creating this Task does not authorize starting it.
