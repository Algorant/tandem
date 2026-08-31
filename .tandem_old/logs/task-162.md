---
id: task-162
type: task
title: "Complete TUI chrome, text, state, and visibility boundaries"
priority: "medium"
parentId: "task-146"
blockers: ["task-161"]
references: ["task-145"]
relatedFiles: ["plan/refactor_spec.md", "tandem/src/tui/mod.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/text.rs", "tandem/src/tui/"]
tags: ["tui", "rust", "architecture", "refactor"]
createdAt: "2026-07-22T20:43:07Z"
updatedAt: "2026-07-28T23:44:46Z"
accord:
  status: "accepted"
  assignee: "worker-task-162-d5fbdf90"
  claimedAt: "2026-07-28T23:32:21Z"
  deliveredAt: "2026-07-28T23:44:23Z"
  deliverables: ["Commit 5a05b319cdf34b1f7851d3b2089b2b637b0135a2", "Commit d419928", "tandem/src/tui/chrome.rs", "tandem/src/tui/text.rs", "tandem/src/tui/state.rs", "tandem/src/tui/mod.rs"]
  validation:
    commands: ["Independent cargo fmt --check passed", "Independent strict Clippy passed", "Independent cargo test passed: 206 unit + 6 real-command tests", "Direct just dev inspection in Herdr tab 2 passed for Board, Logs, Rules, Decisions, and Help chrome", "PTY launch/quit smoke passed", "git diff --check passed"]
  constraints: ["Retained Review-only lint exceptions are documented pending a separate product decision"]
  summary: "Accepted revised delivery after architecture review, independent automated validation, direct Herdr tab-2 inspection of all top-level views and Help chrome, and clean integration."
  evidence: ["tui/mod.rs production/test boundary now at line 369", "Shared Markdown rendering has demonstrated Board/Logs/Review/Decisions call sites", "Worker checkout clean"]
  filesChanged: ["tandem/src/tui/mod.rs", "tandem/src/tui/state.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/text.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/review.rs"]
  reviewer: "orchestrator"
  updatedAt: "2026-07-28T23:44:41Z"
assignee: "worker-task-162-d5fbdf90"
completedAt: "2026-07-28T23:44:46Z"
completion:
  summary: "Completed TUI chrome, shared text, and transient state boundaries; integrated as b67ca4b with tui/mod.rs reduced to a focused production wiring root."
  filesChanged: ["tandem/src/tui/mod.rs", "tandem/src/tui/state.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/text.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/review.rs"]
  validation: "Direct just dev inspection passed across Board, Logs, Rules, Decisions, and Help; cargo fmt, strict Clippy, 206 unit tests, 6 real-command tests, PTY smoke, and diff checks passed."
  reviewer: "orchestrator"
---

## Description

## Objective

Finish the TUI architecture after Board extraction by assigning remaining shared UI behavior to cohesive owners and tightening the root aggregate.

## Scope

- Extract shared tabs/header/footer/help and hit geometry to `tui/chrome.rs` only where ownership is genuinely shared.
- Extract shared Markdown/detail/wrapping helpers to `tui/text.rs` only after multiple call sites prove the boundary.
- Group feature/modal/reload state in behavior-preserving changes where it reduces the oversized aggregate without exposing fields broadly.
- Keep Rules and Decisions feature-vertical; retain `review.rs` until a separate product decision.
- Replace broad imports and temporary `pub(crate)` visibility with private/`pub(super)` APIs where possible.
- Reduce `tui/mod.rs` to high-level aggregate/event-loop wiring, targeting roughly 500 lines without treating size as an automated rule.

## Acceptance criteria

- TUI module ownership is explicit without artificial one-file-per-concept fragmentation.
- Keybindings, help/footer actions, hit regions, rendering, themes, modal/focus behavior, and feature selection remain unchanged.
- Existing TUI unit/TestBackend tests, full tests, real-command tests, PTY checks, formatting, and strict Clippy pass.
- Genuine human `just dev` validation approves every top-level view and representative narrow/wide terminal behavior.
- Temporary TUI lint expectations and migration-only visibility are removed.
- No protocol/product redesign, feature removal, release, or push occurs.

Creating this Task does not authorize starting it.
