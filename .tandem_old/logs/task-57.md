---
id: task-57
type: task
title: "Rework TUI help popup organization and visual design"
priority: "medium"
relatedFiles: ["tandem/src/tui.rs", "tandem/plan/spec.md"]
tags: ["tui", "help", "ux", "design"]
createdAt: "2026-06-29T02:03:28Z"
updatedAt: "2026-06-29T17:54:47Z"
accord:
  status: "accepted"
  deliveredAt: "2026-06-29T03:06:15Z"
  summary: "Reworked the TUI help popup into grouped, styled sections for Global, Navigation, Board, Validation, Logs, Rules, Decisions, and Prompts, with command text updated to current behavior. Added focused assertions for help organization/content."
  evidence: ["cd tandem && cargo fmt --check && cargo test (74 passed)", "Commit 52b7152 on branch herd-help-footer-56-57"]
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "tui"
  updatedAt: "2026-06-29T17:22:38Z"
review.decidedAt: "2026-06-29T17:22:38Z"
review.reviewer: "tui"
review.status: "accepted"
completedAt: "2026-06-29T17:54:47Z"
completion:
  summary: "Accepted after review. Help popup organization and visual grouping are good."
  filesChanged: ["tandem/src/tui.rs"]
  validation: "Human review accepted; merged cargo test suite passed."
---

## Description

The current TUI help popup is visually messy and hard to scan. Rework the help surface so commands are grouped and visually separated in a clearer, more intentional way.

Scope:
- Group commands by pane/view and/or by function, such as navigation, Board actions, Validation actions, Logs, Rules, Decisions, editing, and global commands.
- Improve visual hierarchy and spacing so the popup feels designed rather than a flat scattered list.
- Keep keyboard-first discoverability, but avoid overwhelming the user with unstructured command noise.
- Ensure help text matches current implemented behavior, especially after recent footer/action/filter changes.
- Keep styling consistent with the minimalist Tandem theme direction.

Acceptance:
- Help popup is easier to scan at common terminal sizes.
- Commands are grouped with clear headings or separators.
- View-specific commands are distinguishable from global commands.
- Visual spacing/contrast is improved without becoming jarring.
- Help content does not advertise unavailable or stale actions.
- Add focused tests/snapshot-style assertions where practical.
