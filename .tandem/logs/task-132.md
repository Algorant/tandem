---
id: task-132
type: task
title: "Add collapsible subtask hierarchy to the default Board"
priority: "high"
references: ["decision-4", "task-101", "task-129"]
relatedFiles: ["tandem/src/tui.rs", "tandem/src/tui/review.rs", "tandem/plan/spec.md", "docs/tui/index.md"]
tags: ["tui", "subtasks", "board", "ux", "keyboard", "mouse"]
createdAt: "2026-07-14T13:05:39Z"
updatedAt: "2026-07-14T20:01:43Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-14T13:08:44Z"
  deliveredAt: "2026-07-14T19:56:17Z"
  deliverables: ["Collapsible recursive State Board hierarchy with keyboard and mouse interaction", "Quiet Option C rows without task IDs, parent arrows, or SUB chips", "Cross-state labels, descendant rollups, filter ancestor retention, reload/selection preservation, and legacy/generic-parent handling", "Updated tandem/plan/spec.md and docs/tui/index.md", "One-command Git-local preview route configured for `just dev`"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml -- --check — passed", "cargo test --manifest-path tandem/Cargo.toml — 146 passed", "cargo build --manifest-path tandem/Cargo.toml — passed", "git diff --check — passed", "just site-build — 15 pages built", "cd site && bun run check:links — 602 internal links passed", "Git-local `just dev` route smoke-rendered the safe hierarchy fixture", "Worker git status clean at ed72086fd04846e67ee082942b8fc8c1e7d6e30f"]
  summary: "Accepted after explicit human visual validation of the Option C State Board hierarchy. Implementation was fast-forwarded to main at ed72086; procedural IDs/SUB labels remain hidden, hierarchy is compact and collapsible, and post-merge formatting plus all 146 Rust tests pass."
  evidence: ["Human requested Option C after first preview: hide row identity and redundant procedural labels", "Parent inspected State Board traversal/rendering/interaction code, focused tests, specification, and public documentation", "Preview route file has two valid paths to the delegated manifest and safe fixture"]
  filesChanged: ["tandem/src/tui.rs", "tandem/plan/spec.md", "docs/tui/index.md"]
  reviewer: "user-and-parent-orchestrator"
  updatedAt: "2026-07-14T20:01:35Z"
completedAt: "2026-07-14T20:01:43Z"
completion:
  summary: "Shipped collapsible inline State Board hierarchy with quiet Option C rows, compact guides, keyboard/mouse expansion, cross-state context, filter/reload behavior, documentation, and one-command delegated visual previews."
  filesChanged: ["justfile", "tandem/src/tui.rs", "tandem/plan/spec.md", "docs/tui/index.md"]
  validation: "Explicit human visual approval; accepted accord; post-merge cargo fmt --check and all 146 Rust tests passed; docs built 15 pages and 602 internal links passed."
  reviewer: "user-and-parent-orchestrator"
---

## Description

Integrate first-class subtask navigation into the default State Board so subtasks are not presented as unrelated flat peers and do not require a separate subtask-oriented view.

Product direction:
- The default Board supports task hierarchy directly.
- Task-to-task children are collapsed/hidden by default.
- Parent rows expose a concise child/descendant indicator and an inline expand/collapse affordance (`Enter` or a similarly discoverable control after reconciling the existing row-preview binding).
- Expanding a parent reveals its active descendants inline with clear nesting and the established compact `SUB`/state and `<parent> → <child>` language.
- Epic Board may remain an epic-specific arrangement, but it must not be the only way to discover or navigate subtasks. Do not add a separate Subtask Board.

Acceptance criteria:
- Default State Board no longer renders task-to-task children as independent flat root rows.
- Root tasks and tasks with generic non-task parents remain normal Board rows.
- Parent rows clearly indicate collapsed active/logged descendant counts without clutter.
- Keyboard and mouse users can expand/collapse recursively; selection, scrolling, reload, and hot-reload preserve sensible hierarchy state.
- Resolve the current `Enter` preview behavior deliberately: combine it coherently with hierarchy expansion or choose/document another concise binding.
- Filtering/search cannot silently hide matching descendants: retain/reveal the necessary ancestor path or provide equally clear matched-child context.
- Define how state tabs/counts treat hidden children and how expanded children in other workflow states are labeled.
- Completed descendants remain in Logs and contribute rollups/context without appearing as active rows.
- Existing flat-ID children remain supported through canonical `parentId`.
- Generic decision/custom-document parent relationships are not labeled as subtasks.
- Add focused hierarchy, state/filter, input/navigation, reload, legacy-flat, generic-parent, rendering, and narrow-width tests.
- Update TUI/public documentation and require human visual/UX validation before acceptance.
