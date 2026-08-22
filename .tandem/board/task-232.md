---
id: task-232
type: task
title: "Wrap the TUI draw call in synchronized output to stop clear-then-repaint flicker"
state: "in-progress"
priority: "medium"
effort: "small"
relatedFiles: ["tandem/src/tui/terminal.rs", "tandem/src/tui/mod.rs"]
tags: ["tui", "keyboard", "smoke"]
createdAt: "2026-08-21T03:28:29Z"
updatedAt: "2026-08-22T15:21:42Z"
accord:
  status: "claimed"
  assignee: "worker-task-232-75eae6d3"
  claimedAt: "2026-08-22T15:21:42Z"
  updatedAt: "2026-08-22T15:21:42Z"
assignee: "worker-task-232-75eae6d3"
---

## Description

Wrap the single Ratatui draw call in terminal mode 2026 synchronized output so Ghostty cannot present a screen that has been cleared but not yet repainted.

## Investigation

Captured `tandem tui` 0.10.3 raw pty output with byte-level timing, first inside a Herdr pane via `script`, then in a pty sized once before exec to remove layered-pty interference.

- Per-key navigation is already safe. In a fixed 120x46 pty, Down/Down/Up emitted 246, 255, and 277 bytes, each a single atomic write with no screen clear. The original premise, that the first navigation input sends a flicker-inducing incremental update, does not reproduce.
- Two earlier observations were measurement artifacts of the layered pty, not app behavior: a second full repaint about 0.44 s after launch, and an extra empty draw after the first keypress. Both came from a SIGWINCH caused by `script` starting at 66x44 inside a 69x46 pane. Neither occurs in a clean pty.
- The only reproducible flicker mechanism is clear-then-repaint. Forcing a mid-session resize produced a bare `\x1b[2J` as its own 4-byte write, followed 26 ms later by a 10.9 KB repaint split across three writes at crossterm's 4096-byte buffer boundary. Startup has the same shape: 52 bytes containing `\x1b[2J`, then 11 KB across three writes. The screen is explicitly blanked and then repainted over four separate writes, and the terminal may present anything in that window.
- The clear originates in ratatui `Terminal::autoresize()` -> `resize()` -> `clear()`, which runs inside `Terminal::draw`. Synchronized markers placed around the draw call therefore cover both the clear and the repaint.

Not confirmed: that this resize path is what the user actually sees in Ghostty. The working theory is that Ghostty issues a size change shortly after launch and the resulting flash gets attributed to whichever key was pressed near it. Verification is manual and follows the change.

## Scope

`crossterm` 0.28.1 is already in the lock file, so `BeginSynchronizedUpdate` and `EndSynchronizedUpdate` are available.

- Wrap the single draw call site at `tandem/src/tui/mod.rs:342` in `BeginSynchronizedUpdate` / `EndSynchronizedUpdate`.
- End the synchronized update even when the draw returns an error, so the terminal is never left mid-update.
- Add one focused test asserting the emitted stream contains the `\x1b[?2026h` / `\x1b[?2026l` pair around frame output.
- Validate manually in Ghostty afterward, including a deliberate window resize.

Out of scope: wrapping incremental per-key draws separately, changing the redraw loop, and touching `terminal.clear()` in `enter()` or `resume_after_editor()`.

Reference: https://ghostty.org/docs/help/synchronized-output
