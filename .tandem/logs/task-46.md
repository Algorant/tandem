---
id: task-46
type: task
title: "Unify TUI top header/navigation bar"
priority: "low"
createdAt: "2026-06-28T18:30:22Z"
updatedAt: "2026-06-29T02:06:34Z"
accord:
  status: "accepted"
  deliveredAt: "2026-06-29T01:52:35Z"
  summary: "Accepted after visual review of the reworked header/navigation layout."
  evidence: ["commit 4859766 on branch rework-tui-layout-46-49-50", "cd tandem && cargo fmt --check && cargo test (71 passed)", "Added focused unit coverage for header tab text avoiding ambiguous '1 Board 2' style labels."]
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "human"
  updatedAt: "2026-06-29T02:06:29Z"
completedAt: "2026-06-29T02:06:34Z"
completion:
  summary: "Accepted after visual review. Reworked header/navigation layout is good: shortcut, label, and count are clearer and the header uses space better."
  filesChanged: ["tandem/src/tui.rs"]
  validation: "Human visual review accepted; merged cargo test suite passed."
---

## Description

Problem: the TUI top bar repeats section/navigation information in multiple places, making the header feel redundant and cluttered.

Desired outcome: replace the repeated navigation/header rows with one cohesive header bar that shows primary sections once, makes the current section obvious, and keeps any secondary context compact and non-duplicative.

Acceptance criteria:
- Header has a single clear navigation model.
- No repeated section names/counts across adjacent header rows.
- Current section is visually obvious.
- Secondary context, if present, is compact and does not repeat primary navigation.

## Feedback

### 2026-06-29 — Rework requested

Human visual review requested another pass on the unified top header/navigation bar.

- Current header is too left-heavy and cramped.
- Consider centered or better distributed navigation now that the left-justified layout may not be the best fit.
- Use the horizontal space more intentionally.
- The TUI cannot directly control terminal font size, but spacing, emphasis, and grouping should make the header feel clearer and less cramped.
- Labels like `1 Board 12` and `2 Logs 43` are confusing because the shortcut number and item count read as the same kind of number.
- Separate shortcut, view label, and count more clearly, for example `[1] Board (12)` or centered tabs with muted count badges.
- Screenshot context showed the current top nav above the Validation board with `1 Board 12  2 Logs 43  3 Rules 4  4 Decisions 0` and the ambiguity was noticeable.
