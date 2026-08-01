---
id: task-86
type: task
title: "Improve Decision TUI rendering and board classification"
priority: "high"
parentId: "task-81"
references: ["task-85"]
relatedFiles: ["tandem/src/tui.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/theme.rs", "docs/tui/index.md"]
tags: ["tui", "decision", "adr", "ux"]
createdAt: "2026-07-01T18:05:52Z"
updatedAt: "2026-07-01T23:05:33Z"
accord:
  status: "accepted"
  assignee: "shep-decision-tui"
  claimedAt: "2026-07-01T18:08:12Z"
  deliveredAt: "2026-07-01T23:05:07Z"
  deliverables: ["Merged commits on main: abdae1d task-86: improve decision TUI rendering; f0b30c3 task-86: refine decision list layout; 82a347c task-86: compact decision list expansion", "Decision detail pane preserved as approved", "Decision list uses compact rows, minimal selection, and Enter expansion"]
  validation:
    commands: ["Human visual validation approved by Algorant", "cargo fmt --manifest-path tandem/Cargo.toml --check: pass", "cargo test --manifest-path tandem/Cargo.toml --quiet: 115 passed", "bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts: pass", "git diff --check: pass"]
  summary: "Accepted after human visual validation approved the compact Decisions list rework and merged commits passed validation."
  evidence: ["Approved in conversation after just dev review"]
  filesChanged: ["tandem/src/tui.rs", "tandem/src/tui/decisions.rs"]
  reviewer: "Algorant/orchestrator"
  updatedAt: "2026-07-01T23:05:19Z"
completedAt: "2026-07-01T23:05:33Z"
completion:
  summary: "Completed Decision TUI rendering and board classification work after approved rework. Main now includes compact Decisions list rows, minimal selection, Enter expansion, and approved detail pane rendering."
  validation: "Human visual validation approved by Algorant. Automated validation passed: cargo fmt --check; cargo test --quiet 115 passed; bun --check pi-tandem tests; git diff --check."
  reviewer: "Algorant/orchestrator"
---

## Description

Improve TUI treatment of Decision documents: Decisions pane should render ADR-compatible metadata/body cleanly, decisions should not depend on task workflow state, and board/classification code should not surface decision documents as `unfiled` board items. Keep terminology as Decision/Decisions.
