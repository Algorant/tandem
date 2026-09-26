---
id: task-45-1
type: task
title: "Add `s`-cycled Board sort modes (default ID ascending)"
priority: "medium"
parentId: "task-45"
tags: ["tui", "keyboard"]
accord:
  status: "accepted"
  acceptance: ["`s` cycles ID asc → ID desc → newest created → recently updated → priority; default ID ascending; session-only.", "Sort applies at all hierarchy levels in State and Epic Board with state grouping kept; active mode shown in chrome/status and `s` listed in key reference.", "task-36 newest-first Board tests are replaced by per-mode tests including child-level order."]
  claimedAt: "2026-09-26T19:25:07Z"
  deliveredAt: "2026-09-26T19:33:24Z"
  summary: "Implemented session-only five-mode Board sorting with numeric ID comparison, key binding, chrome/status label, preserved selection, and hierarchy-level ordering in State and Epic projections."
  evidence: ["cargo test --manifest-path tandem/Cargo.toml passes; dedicated tests exercise each mode on Epic children and Subtasks in State/Epic arrangements, numeric ID comparison and cycle wrap.", "Rendered release-built just dev sandbox in Herdr pane w97:p2: footer showed sort ID ascending; pressing s showed sort ID descending. Epic Board Enter retains inline-preview behavior; no forced Epic expansion exists."]
  updatedAt: "2026-09-26T19:35:14Z"
createdAt: "2026-09-26T19:23:07Z"
updatedAt: "2026-09-26T19:35:14Z"
assignee: "worker-task-45-d8de4d68"
archivedAt: "2026-09-26T19:35:14Z"
resolution:
  outcome: "completed"
---

