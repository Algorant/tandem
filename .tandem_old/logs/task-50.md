---
id: task-50
type: task
title: "Rework Rules pane into Board-style category view"
priority: "low"
createdAt: "2026-06-28T18:30:49Z"
updatedAt: "2026-06-29T17:54:47Z"
accord:
  status: "accepted"
  deliveredAt: "2026-06-29T03:05:35Z"
  summary: "Adjusted only selected Always rule category styling so it uses selected foreground on selected background with bold, keeping the neutral/subtle palette while reading active instead of disabled. Other category highlight treatments are unchanged. Added focused unit coverage. Commit 51d2109 on herd-visual-rework-49-50."
  evidence: ["cd tandem && cargo fmt --check && cargo test (74 passed)", "Commit 51d2109 Refine logs and rules visual rework"]
  filesChanged: ["tandem/src/tui/theme.rs"]
  reviewer: "tui"
  updatedAt: "2026-06-29T17:13:24Z"
review.decidedAt: "2026-06-29T17:13:24Z"
review.reviewer: "tui"
review.status: "accepted"
completedAt: "2026-06-29T17:54:47Z"
completion:
  summary: "Accepted after visual review. Rules pane category view and selected Always styling are good."
  filesChanged: ["tandem/src/tui/rules.rs", "tandem/src/tui/theme.rs"]
  validation: "Human visual review accepted; merged cargo test suite passed."
---

## Description

Problem: the Rules pane is vertically split and does not match the desired interaction model. It should behave more like the Board pane / Brainfile TUI rules view.

Desired outcome: show one main horizontal pane for one rule category at a time, with category navigation across Always, Never, Prefer, and Context.

Acceptance criteria:
- Rules categories are navigable like Board sections/subboards.
- Only one rule category is shown at a time.
- Remove the unnecessary vertical split.
- Rule list is readable and minimal.
- Enter expands or shows more detail for the selected rule when detail exists.
- Empty states are simple, e.g. No always rules defined. Press n to add one.
- Footer commands are scoped to Rules, e.g. h/l switch type, n new, e edit, d delete.

## Feedback

### 2026-06-29 — Rework requested

Human visual review says the current Rules pane is a step in the right direction, but needs another visual design pass.

- Brainfile's rules view has a clearer and nicer design language to use as reference.
- Category navigation should feel spacious and minimalist, not like generic cramped tabs.
- Selected categories should use distinct color coding:
  - Always: neutral/subtle
  - Never: red
  - Prefer: yellow
  - Context: purple
- Shortcut numbers should be visually secondary to the category labels.
- The selected category should read as an intentional pill/accent.
- Preserve Tandem behavior and v0 scope, but move the visual language closer to the Brainfile reference screenshots.

### 2026-06-29 — Rework requested again

Human visual review says Rules is almost perfect after the rework, with one remaining styling issue.

- The selected/highlighted `Always` category looks greyed out and is not easily discernible.
- The other category highlight treatments are good.
- Adjust selected `Always` styling so it reads clearly as active/selected while remaining neutral/subtle.
- Avoid making selected `Always` look disabled.
