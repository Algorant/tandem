---
id: task-181
type: task
title: "Add supported decision correction and withdrawal workflows"
priority: "medium"
relatedFiles: ["tandem/src/main.rs", "tandem/src/tui.rs", "protocol/plan/spec.md", "tandem/plan/spec.md"]
tags: ["protocol", "decisions", "cli", "tui"]
createdAt: "2026-07-24T00:51:28Z"
updatedAt: "2026-07-24T03:58:58Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-07-24T01:17:48Z"
  deliveredAt: "2026-07-24T03:25:08Z"
  deliverables: ["tandem decision update <id> --title/--body/--status", "tandem decision withdraw <id> --reason preserving decision record with withdrawnAt and withdrawalReason", "decision.withdrawn event and withdrawn ADR status", "protocol/CLI/TUI guidance updates"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test (162 passed)", "Temporary workspace CLI smoke: add → update → withdraw; verified amended body, withdrawn status, reason, and timestamp", "git diff --check"]
  summary: "User accepted supported, auditable decision update and withdrawal workflows."
  evidence: ["The TUI Decisions view now directs users to supported CLI update/withdraw commands rather than manual decision-file editing."]
  reviewer: "user"
  updatedAt: "2026-07-24T03:58:47Z"
completedAt: "2026-07-24T03:58:58Z"
completion:
  summary: "Added supported decision update and withdrawal CLI workflows with auditable withdrawn metadata/events and TUI guidance."
  filesChanged: ["tandem/src/main.rs", "tandem/src/tui.rs", "protocol/plan/spec.md", "tandem/plan/spec.md"]
  validation: "cargo fmt --check; cargo test (162 passed); disposable workspace add → update → withdraw smoke"
  reviewer: "user"
---

## Description

Design and implement supported Tandem workflows for decisions created or recorded in error. Cover amendment/edit behavior, explicit withdrawal or deletion semantics with appropriate auditability, and supersession guidance; expose the chosen behavior through CLI and TUI without treating decisions as ordinary Task lifecycle state. Avoid requiring users to manually edit or remove decision Markdown for normal correction paths.
