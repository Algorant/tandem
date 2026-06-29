---
id: task-43
type: task
title: "Finalize TUI keybinding and help discoverability"
state: "validation"
priority: "medium"
parentId: "task-9"
relatedFiles: ["tandem/src/tui.rs", "tandem/plan/spec.md"]
tags: ["tui", "keyboard", "help", "ux"]
createdAt: "2026-06-28T17:29:46Z"
updatedAt: "2026-06-29T00:19:14Z"
accord:
  status: "delivered"
  deliveredAt: "2026-06-29T00:19:14Z"
  deliverables: ["code:tandem/src/tui.rs:accurate footer/status/help keybinding surfaces", "docs:tandem/plan/spec.md:keybinding/help behavior if user-visible semantics change"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test", "cd tandem && cargo build"]
  constraints: ["Keep fixed v0 defaults; custom keymap config remains deferred.", "Do not advertise unavailable actions."]
  summary: "Finalized TUI help/footer discoverability by moving dense persistent hints into compact contextual footer text while keeping full help under ?. Updated tests for compact contextual footer behavior."
  evidence: ["Commit 1d95ef9 Tighten TUI footer hints", "cd tandem && cargo fmt --check && cargo test (61 passed)"]
  filesChanged: ["tandem/src/tui.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/decisions.rs", "tandem/plan/spec.md"]
  updatedAt: "2026-06-29T00:19:14Z"
---

## Description

Follow-up split from task-9.

Scope:
- Reconcile footer, status, and help-table text after the runtime interaction and rendering slices.
- Ensure Board, Validation, Logs, Rules, and Decisions views expose current keyboard shortcuts accurately.
- Keep fixed v0 defaults; custom keymap config remains deferred.
- Preserve vim-style and conventional navigation where already implemented.

Traceability:
- Parent tracker: task-9.
- Replaces open subtask task-9-4.

Acceptance:
- Help text matches implemented behavior and no longer advertises unavailable actions.
- User-visible keybinding changes are reflected in tandem/plan/spec.md if needed.
