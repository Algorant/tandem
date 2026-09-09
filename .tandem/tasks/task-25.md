---
id: task-25
type: task
title: "Show Accord attempt/rework/discarded counts in a reachable TUI detail panel"
state: "in-progress"
priority: "medium"
effort: "small"
references: ["task-23"]
relatedFiles: ["tandem/src/tui/logs.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/input.rs"]
tags: ["tui", "accord", "delegation"]
accord:
  status: "claimed"
  acceptance: ["Logs detail (`detail_lines_for_log`) renders `Attempts: N`, `Reworks: N`, `Discarded: N` when any count is non-zero, using `logs::accord_counts` from the Task's events.", "The Board detail panel is reachable from the keyboard on the Board view and its toggle is listed in the Board footer; or, if the panel is intentionally retired, `draw_detail`/`toggle_board_detail`/`HitAction::ToggleBoardDetail` and the `board_detail_shows_accord_attempt_counts_when_present` test are removed and counts are surfaced in the Board row or another reachable place instead.", "Verified by rendering in a Herdr pane, not only by unit test."]
  claimedAt: "2026-09-09T15:05:20Z"
  validation: ["$ cargo test -p tandem", "Live: in a throwaway workspace claim/deliver/rework/release --disposition discarded/claim/deliver/complete a Task, open `tandem tui`, press 2, select it; the Log detail shows Attempts: 2, Reworks: 1, Discarded: 1. On the Board, select an active reworked Task and open its detail from the keyboard; the same labels appear."]
  updatedAt: "2026-09-09T15:05:20Z"
createdAt: "2026-09-09T14:51:01Z"
updatedAt: "2026-09-09T15:05:20Z"
assignee: "Algorant"
---

## Description


Found while verifying task-23 against the release binary (0.12.4 dev build, 2026-09-09) in a fresh workspace via a Herdr pane.

- Log detail for an archived Task with attemptCount 2 / reworkCount 1 / discardedCount 1 shows the raw event timeline but no count lines. `detail_lines_for_log` does not call `accord_counts`.
- The Board detail panel (`draw_detail`) is the only place counts render. `show_board_detail` defaults to false and `toggle_board_detail` is reachable only through `HitAction::ToggleBoardDetail`, registered against the footer label `"Tab board"`. No Board footer string contains that label (Tab is bound to `next_state` on the Board), so the hit never registers and the panel cannot be opened. That is pre-existing, but it means task-23's TUI acceptance is met only in unreachable code.

JSON/CLI surfaces from task-23 and task-24 verified correct; this is TUI-only and does not block the Pi adapter work.

