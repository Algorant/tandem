---
id: task-33
type: task
title: "Refine Board layout, headers, and inline task expansion"
priority: "high"
tags: ["tui", "board"]
createdAt: "2026-06-28T14:35:32Z"
updatedAt: "2026-06-28T15:31:34Z"
accord:
  status: "accepted"
  assignee: "ui-tui-design-batch"
  claimedAt: "2026-06-28T14:58:57Z"
  deliveredAt: "2026-06-28T15:05:19Z"
  summary: "Implemented compact Board header, hidden-by-default detail pane, Enter inline previews, chip-before-title rows, and suppressed zero-progress chips; validation passed."
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "ivan"
  note: "Human visual check accepted the Enter inline expansion and Tab detail-pane flow; minor refinements deferred."
  updatedAt: "2026-06-28T15:31:34Z"
completedAt: "2026-06-28T15:31:34Z"
completion:
  summary: "Accepted Board layout/header/inline expansion work; Enter expands rows and Tab toggles the detail pane as intended."
  filesChanged: ["tandem/src/tui.rs"]
  validation: "cargo fmt --check; cargo test; cargo build passed; human visual check accepted core flow"
  reviewer: "ivan"
---

## Description

Design and implement a calmer Brainfile-inspired Board surface.\n\nScope:\n- Use one large Board pane by default; the persistent detail pane should be hidden by default and toggleable with a key.\n- Enter on a task expands/collapses an inline preview under that row instead of forcing attention into a separate detail pane.\n- Expanded preview should prioritize: tags, a concise summary/body excerpt, and related files. Keep it at-a-glance, not a full metadata dump.\n- Remove unhelpful todo-row progress/accord counters such as 0/6 when no progress has occurred or when the signal does not change next action.\n- Move row chips before the title in a stable order: selection marker, priority/status chips, title, muted id.\n- Fix selected-row styling so chips and titles remain readable when highlighted; prefer a left accent/marker over full-row inversion.\n- Combine the duplicated top header/view-selector bands into one compact header. State tabs should remain local to the Board list.\n- Reduce heavy borders/chrome where possible so the Board feels like a sparse workspace, not nested boxes.\n\nDesign guardrails:\n- Board rows are for scanning and next action only.\n- Detail metadata belongs in inline expansion or a toggled detail surface, not always-visible rows.\n- Any chip/badge must be legible in selected and unselected states.\n- Do not copy Brainfile exactly; use it as the initial design-language reference.
