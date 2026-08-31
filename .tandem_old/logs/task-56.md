---
id: task-56
type: task
title: "Keep TUI footer styling uniform across panes"
priority: "medium"
relatedFiles: ["tandem/src/tui.rs", "tandem/src/tui/theme.rs"]
tags: ["tui", "theme", "footer", "ux", "bug"]
createdAt: "2026-06-29T02:02:34Z"
updatedAt: "2026-06-29T17:54:47Z"
accord:
  status: "accepted"
  deliveredAt: "2026-06-29T03:06:11Z"
  summary: "Implemented neutral footer rendering so ordinary hotkey hints use the base text style across views, while status suffixes are styled independently and prompt/status warnings remain isolated. Added regression coverage for status-tone leakage."
  evidence: ["cd tandem && cargo fmt --check && cargo test (74 passed)", "Commit 52b7152 on branch herd-help-footer-56-57"]
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "tui"
  updatedAt: "2026-06-29T17:22:35Z"
review.decidedAt: "2026-06-29T17:22:35Z"
review.reviewer: "tui"
review.status: "accepted"
completedAt: "2026-06-29T17:54:47Z"
completion:
  summary: "Accepted after review. Footer/hotkey styling remains uniform and neutral across panes."
  filesChanged: ["tandem/src/tui.rs"]
  validation: "Human review accepted; merged cargo test suite passed."
---

## Description

The TUI footer/hotkey bar should keep a uniform neutral/minimal style across all panes and view switches. A visual bug was observed where switching to the Logs pane made the footer revert to an ugly green color, and that color was retained when switching back to Board until additional actions caused it to return to neutral foreground text.

Scope:
- Make footer/hotkey styling consistent across Board, Logs, Rules, Decisions, and any prompt/status states.
- Footer style should not inherit stale status tone/color from previous panes unless there is an intentional warning/error/success status.
- Prefer a neutral foreground/minimal style for ordinary hotkey hints.
- Ensure view switching does not leave stale color state behind.
- If status messages need color, isolate that styling from persistent hotkey/help text so normal footer hints remain visually stable.

Acceptance:
- Switching Board -> Logs -> Board does not change footer hotkey color unexpectedly.
- Footer hotkey style remains uniform across panes in normal state.
- Any warning/success/error status styling is deliberate and clears predictably.
- Add focused regression coverage for footer style/tone selection if practical.
