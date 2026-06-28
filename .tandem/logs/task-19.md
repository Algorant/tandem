---
id: task-19
type: task
title: "Tighten TUI keyboard and focus semantics"
priority: "high"
references: ["task-11"]
relatedFiles: ["tandem-tui/src/tui.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md"]
tags: ["ui", "tui", "keyboard", "focus"]
createdAt: "2026-06-28T00:17:02Z"
updatedAt: "2026-06-28T01:34:29Z"
accord:
  status: "accepted"
  assignee: "herd:tui-keyboard-focus"
  claimedAt: "2026-06-28T00:35:13Z"
  deliveredAt: "2026-06-28T00:59:38Z"
  deliverables: ["code:tandem-tui/src/tui.rs:local-only navigation semantics", "docs:tandem-tui/README.md:updated controls"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check && cargo test && cargo build", "PTY smoke for 1..5 top-level switching, hjkl local movement, and Tab no-op/hint where no pane focus exists"]
  constraints: ["Do not make h/l switch top-level views by default."]
  summary: "Tightened TUI keyboard/focus semantics: numeric-only keyboard view switching, local h/j/k/l navigation, local Tab focus/no fallback, plus docs/tests updates."
  evidence: ["Commit dda4c60 on branch tandem-task19-keyboard-focus.", "Validation passed: cd tandem-tui && cargo fmt --check && cargo test && cargo build; git diff --check.", "PTY smoke passed: target/debug/tdm tui scripted 1..5 top-level switching, h/j/k/l local movement, and Tab/BackTab focus/no-fallback sequences exited rc=0."]
  filesChanged: ["tandem-tui/src/tui.rs", "tandem-tui/src/tui/decisions.rs", "tandem-tui/src/tui/rules.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  reviewer: "pi"
  updatedAt: "2026-06-28T01:34:29Z"
completedAt: "2026-06-28T01:34:29Z"
completion:
  summary: "Integrated tightened TUI keyboard and focus semantics; validation passed after merge."
  validation: "cargo fmt --check; cargo test; cargo build"
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
