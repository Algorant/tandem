---
id: task-21
type: task
title: "Open selected Tandem items in $EDITOR from TUI"
state: todo
priority: "high"
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/plan/spec.md"]
tags: ["ui", "tui", "editor", "workflow"]
createdAt: "2026-06-28T00:17:03Z"
updatedAt: "2026-06-28T01:43:16Z"
accord:
  status: "claimed"
  assignee: "pi"
  claimedAt: "2026-06-28T01:43:16Z"
  deliverables: ["code:tandem-tui/src/tui.rs:external editor action", "docs:tandem-tui/README.md:editor key and behavior"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check && cargo test && cargo build"]
  constraints: ["Do not corrupt terminal raw mode or alternate screen on editor launch/return."]
  updatedAt: "2026-06-28T01:43:16Z"
---

## Description

Add an external editor action for Tandem TUI editable documents, defaulting to `$EDITOR`.

Context:
- The user expects to open any task or editable item in their editor, usually `nvim` via `$EDITOR`.
- This should work for tasks first and be designed to extend to rules, decisions, and other editable documents.

Acceptance direction:
- Add a keyboard action to open the selected editable document in `$EDITOR` (fall back to a sensible default only if needed).
- Restore terminal state cleanly before launching the editor and return to the TUI afterward.
- Reload the workspace after editor exit and surface parse/write errors safely.
- Support active task documents first; document which editable surfaces are included or deferred.
- Avoid opening generated logs for mutation unless the UX explicitly marks them read-only or confirms intent.
