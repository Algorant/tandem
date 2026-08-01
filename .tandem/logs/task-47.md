---
id: task-47
type: task
title: "Scope persistent TUI footer hints to current context"
priority: "low"
createdAt: "2026-06-28T18:30:49Z"
updatedAt: "2026-06-29T01:37:32Z"
accord:
  status: "accepted"
  deliveredAt: "2026-06-29T00:19:14Z"
  summary: "Accepted after visual review; contextual footer hints are good."
  evidence: ["Commit 1d95ef9 Tighten TUI footer hints", "Added footer_hints_are_contextual_and_compact unit coverage", "cd tandem && cargo fmt --check && cargo test (61 passed)"]
  filesChanged: ["tandem/src/tui.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/decisions.rs"]
  reviewer: "human"
  updatedAt: "2026-06-29T01:37:19Z"
completedAt: "2026-06-29T01:37:32Z"
completion:
  summary: "Accepted after visual review. Persistent footer hints are now contextual and concise enough for the current TUI direction."
  filesChanged: ["tandem/src/tui.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/decisions.rs"]
  validation: "Human visual review accepted; merged cargo test suite passed."
---

## Description

Problem: the bottom command/help bar is too dense and shows a giant jumble of commands even though ? exposes the full key list.

Desired outcome: persistent footer should show only 2-4 highly relevant commands for the active view/pane.

Acceptance criteria:
- Footer shows at most 3-4 commands.
- Commands are contextual to the active pane/view.
- Full keybinding/help remains available via ?.
- Footer feels like quick hints, not a manual.
- Example Board hints: enter detail, m move, a accord, ? help.
- Example Logs hints: enter detail, / search, ? help.
- Example Rules hints: h/l switch type, n new, e edit, d delete.
