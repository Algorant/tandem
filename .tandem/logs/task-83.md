---
id: task-83
type: task
title: "Add Epic TUI rendering and relationship hints"
priority: "high"
parentId: "task-80"
references: ["task-82"]
relatedFiles: ["tandem/src/tui.rs", "tandem/src/tui/theme.rs", "tandem/src/tui/review.rs", "docs/tui/index.md"]
tags: ["tui", "epic", "badges", "relationships"]
createdAt: "2026-07-01T18:05:35Z"
updatedAt: "2026-07-01T23:04:46Z"
accord:
  status: "accepted"
  assignee: "shep-epic-tui"
  claimedAt: "2026-07-01T18:07:48Z"
  deliveredAt: "2026-07-01T23:04:20Z"
  deliverables: ["Merged commits on main: 8fbdc87 Render epic task hints in TUI; e42177e Add grouped Epic Board arrangement", "Epic arrangement groups kind: epic tasks and nests child tasks via parentId", "No noisy P:task-N chips; Enter-expanded rows show relationship context"]
  validation:
    commands: ["Human visual validation approved by Algorant", "cargo fmt --manifest-path tandem/Cargo.toml --check: pass", "cargo test --manifest-path tandem/Cargo.toml --quiet: 115 passed", "bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts: pass", "git diff --check: pass"]
  summary: "Accepted after human visual validation approved the Epic Board UX rework and merged commits passed validation."
  evidence: ["Approved in conversation after just dev review"]
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "Algorant/orchestrator"
  updatedAt: "2026-07-01T23:04:32Z"
completedAt: "2026-07-01T23:04:46Z"
completion:
  summary: "Completed Epic TUI rendering and relationship hints after approved rework. Main now includes grouped Epic Board arrangement and relationship expansion behavior."
  validation: "Human visual validation approved by Algorant. Automated validation passed: cargo fmt --check; cargo test --quiet 115 passed; bun --check pi-tandem tests; git diff --check."
  reviewer: "Algorant/orchestrator"
---

## Description

Add the TUI side of lightweight Epic support: render `kind: epic` tasks with an EPIC badge/marker, keep epics board-visible in normal workflow states, and add useful derived relationship hints such as child counts from `parentId` where practical. Coordinate with the badge-config lane and avoid inventing a dedicated Epic pane.
