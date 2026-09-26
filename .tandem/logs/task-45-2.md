---
id: task-45-2
type: task
title: "Collapse Board hierarchy by default, including with filters"
priority: "medium"
parentId: "task-45"
tags: ["tui"]
accord:
  status: "accepted"
  acceptance: ["Remove the state-mismatch and active-filter forced expansion; only explicit Enter/click opens rows, and toggling reliably collapses with an accurate status message.", "Collapsed rows hiding pane-matching or filter-matching descendants show a compact hint count.", "Tests cover cross-state parents (e.g. in-progress Epic with todo Task on the todo pane) collapsed by default and toggleable, with and without filters; rendered Board verified in a Herdr pane."]
  claimedAt: "2026-09-26T19:29:00Z"
  deliveredAt: "2026-09-26T19:33:28Z"
  summary: "Removed state-mismatch and filter-forced expansion; State Board branches now expand solely on explicit IDs and display matching hidden descendant counts."
  evidence: ["Tests exercise cross-state Epic parent with todo child under unfiltered and tag-filtered todo panes: one collapsed row, one hidden match, explicit expansion reveals child; existing Enter/mouse toggle tests pass.", "Herdr release-built just dev pane w97:p2 ANSI shows collapsed ▸ Epic with '3 matches hidden'; Enter showed ▾ and children #2/#3, second Enter returned ▸ and hidden count. Cross-state visual fixture not separately staged; automated cross-state test covers that path."]
  updatedAt: "2026-09-26T19:35:14Z"
createdAt: "2026-09-26T19:23:07Z"
updatedAt: "2026-09-26T19:35:14Z"
assignee: "worker-task-45-d8de4d68"
archivedAt: "2026-09-26T19:35:14Z"
resolution:
  outcome: "completed"
---

