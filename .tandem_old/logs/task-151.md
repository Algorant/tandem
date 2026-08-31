---
id: task-151
type: task
title: "Establish tui/mod.rs and extract terminal and editor seams"
priority: "high"
parentId: "task-146"
blockers: ["task-150"]
references: ["task-145"]
relatedFiles: ["plan/refactor_spec.md", "tandem/src/tui.rs", "tandem/src/tui/", "tandem/Cargo.toml"]
tags: ["tui", "rust", "refactor", "testing"]
createdAt: "2026-07-22T20:40:56Z"
updatedAt: "2026-07-26T21:54:26Z"
accord:
  status: "accepted"
  assignee: "worker-task-151-6625fd75"
  claimedAt: "2026-07-26T21:47:24Z"
  deliveredAt: "2026-07-26T21:54:11Z"
  deliverables: ["100% path-only tui.rs to tui/mod.rs move commit", "tui/editor.rs command parsing/target/execution seam", "tui/terminal.rs enter/restore/suspend/resume/drop seam", "relocated and expanded focused tests"]
  validation:
    commands: ["cargo fmt --check", "170 unit + 4 executable tests passed", "focused editor and terminal tests passed", "strict Clippy passed", "Worker PTY quit/editor lifecycle checks passed", "Parent Herdr live-vs-dev comparison across Board State/Epic, Logs, Rules, Decisions, help, and editor suspend/resume showed parity", "preview route and disposable tabs cleaned"]
  summary: "Moved the TUI root to tui/mod.rs and extracted behavior-preserving editor and terminal lifecycle seams with colocated tests."
  evidence: ["commits 6e4a5d6 and 9df9724 fast-forward integrated", "no duplicate tui module path", "170 unit + 4 executable tests and strict Clippy passed", "live/dev visible surfaces matched across all top-level views", "alternate-screen cleanup left panes clean after q", "just dev preview route reset"]
  filesChanged: ["tandem/src/tui/mod.rs", "tandem/src/tui/editor.rs", "tandem/src/tui/terminal.rs"]
  reviewer: "parent-orchestrator"
  note: "Reviewed both commits and independently validated code, tests, terminal cleanup, and visual parity. Disposable Herdr tabs compared installed 0.6.5 against routed `just dev` on equivalent fixtures; Board State/Epic, Logs, Rules, Decisions, and help rendered identically. Dev editor suspend/resume via EDITOR=/bin/true returned cleanly with expected status. Route, fixtures, and tabs were cleaned."
  updatedAt: "2026-07-26T21:54:20Z"
assignee: "worker-task-151-6625fd75"
completedAt: "2026-07-26T21:54:26Z"
completion:
  summary: "Established tandem/src/tui/mod.rs and extracted cohesive editor and terminal lifecycle seams without behavior changes. Preserved terminal cleanup/editor suspend-resume, TUI rendering/input/state, and test coverage."
  filesChanged: ["tandem/src/tui/mod.rs", "tandem/src/tui/editor.rs", "tandem/src/tui/terminal.rs"]
  validation: "Parent reviewed both commits, passed formatting, 170 unit tests, 4 executable tests, strict Clippy, focused editor/terminal tests, and confirmed live installed-vs-routed-dev parity in disposable Herdr tabs across all top-level views, help, State/Epic Board, and editor suspend/resume. Preview route and tabs were cleaned."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Prove the campaign's module, visibility, testing, and lint conventions with a low-risk behavior-preserving TUI move.

## Scope

- Move `tandem/src/tui.rs` to `tandem/src/tui/mod.rs` in one dedicated change; never define both module paths simultaneously.
- Extract editor target/command parsing and execution to `tui/editor.rs` with its tests.
- Extract terminal enter/restore/suspend/resume and cleanup safety to `tui/terminal.rs` with its tests.
- Replace broad imports only as needed for explicit, narrow module APIs.
- Preserve the existing TUI aggregate, state shape, input behavior, rendering, keybindings, mouse behavior, themes, and persistent mutations.

## Acceptance criteria

- Movement is attributable and contains no protocol, UI, parser, output, dependency, or product redesign.
- Terminal cleanup remains safe on normal exit, error, panic/drop paths, and editor suspend/resume.
- Existing tests move with implementation without unexplained count reduction.
- Formatting, full tests, focused terminal/editor tests, real-command tests, strict Clippy, PTY validation, and genuine human `just dev` validation pass.
- Temporary lint expectations assigned to these seams are removed.
- No later TUI state/Board split, release, or push occurs.

Creating this Task does not authorize starting it.
