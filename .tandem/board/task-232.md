---
id: task-232
type: task
title: "Eliminate Ghostty TUI flicker with synchronized output"
state: todo
priority: "medium"
effort: "small"
relatedFiles: ["tandem/src/tui/terminal.rs", "tandem/src/tui/mod.rs"]
tags: ["tui", "keyboard", "smoke"]
createdAt: "2026-08-21T03:28:29Z"
updatedAt: "2026-08-21T03:28:29Z"
---

## Description

Wrap each Ratatui frame draw in terminal mode 2026 synchronized output so Ghostty does not render midway through incremental cursor updates. Reproduction: open `tandem tui` in Ghostty and press Down once; a brief screen shift/flicker appears only on the first navigation input. PTY capture confirms Tandem sends a small incremental update rather than a full-screen clear. Use Crossterm `BeginSynchronizedUpdate` and `EndSynchronizedUpdate`, preserve error-safe terminal restoration, add focused automated coverage, and validate manually in Ghostty. Reference: https://ghostty.org/docs/help/synchronized-output
