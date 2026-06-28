---
id: task-19
type: task
title: "Tighten TUI keyboard and focus semantics"
state: "in-progress"
priority: "high"
references: ["task-11"]
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md"]
tags: ["ui", "tui", "keyboard", "focus"]
createdAt: "2026-06-28T00:17:02Z"
updatedAt: "2026-06-28T00:35:13Z"
accord:
  status: "claimed"
  assignee: "herd:tui-keyboard-focus"
  claimedAt: "2026-06-28T00:35:13Z"
  deliverables: ["code:tandem-tui/src/tui.rs:local-only navigation semantics", "docs:tandem-tui/README.md:updated controls"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check && cargo test && cargo build", "PTY smoke for 1..5 top-level switching, hjkl local movement, and Tab no-op/hint where no pane focus exists"]
  constraints: ["Do not make h/l switch top-level views by default."]
  updatedAt: "2026-06-28T00:35:13Z"
---

## Description

Tighten default TUI navigation semantics so local movement stays local and top-level view switching is explicit.

User feedback:
- `h/j/k/l` is overloaded: it sometimes moves inside the current section and sometimes switches top-level Board/Review/Logs/Rules/Decisions views.
- `Tab` can cycle panes where panes exist, but in views without meaningful focusable panes it falls back to switching sections, which feels surprising.

Acceptance direction:
- Make `1`..`5` the only default top-level view-switching shortcuts.
- Make `h/j/k/l` operate only inside the current view/section/subview.
- Make `Tab`/`BackTab` cycle focus only where the current view has meaningful focusable panes; otherwise do nothing or show a status hint, but do not switch top-level views.
- Review Logs/Rules/Decisions behavior for consistency with list/detail/category/body focus.
- Update help/footer text and tests/smokes accordingly.
