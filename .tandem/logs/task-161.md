---
id: task-161
type: task
title: "Extract TUI Board projection, rendering, and hit geometry"
priority: "high"
parentId: "task-146"
blockers: ["task-160"]
references: ["decision-7", "task-134"]
relatedFiles: ["plan/refactor_spec.md", "tandem/src/tui/mod.rs", "tandem/src/tui/board/", "tandem/src/tui/theme.rs"]
tags: ["tui", "hierarchy", "rust", "refactor"]
createdAt: "2026-07-22T20:42:55Z"
updatedAt: "2026-07-28T23:12:16Z"
accord:
  status: "accepted"
  assignee: "worker-task-161-91dea83c"
  claimedAt: "2026-07-28T23:03:16Z"
  deliveredAt: "2026-07-28T23:09:07Z"
  deliverables: ["Commit 375e7b3f09006845ecaf985f49b04c60add636f3", "tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/mod.rs"]
  validation:
    commands: ["Independent parent rerun: cargo fmt --check passed", "Independent parent rerun: cargo clippy --all-targets -- -D warnings passed", "Independent parent rerun: cargo test passed (206 unit + 6 real-command tests)", "Focused Board tests: 16 passed", "git diff --check passed", "Search confirmed Board queries canonical hierarchy APIs and render.rs contains no app/filesystem mutation calls"]
  constraints: ["Human just dev validation across Board modes, keyboard/mouse, narrow layouts, diagnostics, and themes remains required before acceptance/integration"]
  summary: "Accepted after code review, independent automated validation, direct Herdr tab-2 just dev inspection of State/Epic Board modes, expansion/navigation/filter behavior, and clean integration."
  evidence: ["Worker checkout clean", "Move-focused extraction reviewed", "Canonical TaskRole and ParentRelationship values are queried from ProjectHierarchy"]
  filesChanged: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/mod.rs"]
  reviewer: "orchestrator"
  updatedAt: "2026-07-28T23:12:11Z"
assignee: "worker-task-161-91dea83c"
completedAt: "2026-07-28T23:12:16Z"
completion:
  summary: "Extracted canonical hierarchy-backed Board projection and rendering into tui/board and integrated commit 375e7b3 after automated and direct visual validation."
  filesChanged: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/mod.rs"]
  validation: "Direct just dev inspection in Herdr tab 2 passed for State/Epic Board modes, expansion, keyboard navigation, and filtering; cargo fmt, strict Clippy, 206 unit tests, 6 real-command tests, and diff checks passed."
  reviewer: "orchestrator"
---

## Description

## Objective

Move the largest TUI feature seam behind explicit canonical-projection and rendering ownership without changing Board behavior.

## Scope

- Establish a cohesive `tui/board` boundary, beginning with one file and splitting into projection/render leaves only when dependency direction is clear.
- Move Board state/row models, hierarchy-backed projection, expansion, filtering, ordering, ancestor/descendant visibility, cross-state context, details, previews, layout, and frame-local hit regions.
- Query canonical protocol hierarchy results; never rederive Epic/Task/Subtask roles or relationship validity.
- Keep projection free of durable writes and rendering free of app mutations.
- Preserve State and Epic Board modes, selection/reload behavior, narrow-width chips, mouse/keyboard parity, Markdown-ish details, themes, and diagnostics.

## Acceptance criteria

- Searches/review find no duplicate hierarchy inference in TUI Board code.
- Projection and render dependencies are one-way and APIs use private/`pub(super)` visibility where possible.
- Existing Board hierarchy/filter/input/render tests, full tests, real-command tests, PTY checks, formatting, and strict Clippy pass.
- Genuine human `just dev` validation approves all Board modes, key paths, mouse paths, narrow layouts, diagnostics, and themes.
- Temporary lint expectations assigned to Board code are removed.
- No broad framework rewrite, protocol change, release, or push occurs.

Creating this Task does not authorize starting it.
