---
id: task-32
type: task
title: "Update pi-tandem guidance for Validation workflow"
state: "validation"
priority: "med"
relatedFiles: ["extensions/pi-tandem/README.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/plan/todo.md", "extensions/pi-tandem/index.ts"]
tags: ["pi-tandem", "validation", "docs"]
blockers: ["task-28", "task-29", "task-31"]
createdAt: "2026-06-28T13:57:07Z"
updatedAt: "2026-06-28T15:31:34Z"
subtasks:
  - id: task-32-1
    title: "Update prompt guidance so agents deliver work into Validation"
    completed: true
  - id: task-32-2
    title: "Warn agents not to complete human-required validation without approval"
    completed: true
  - id: task-32-3
    title: "Update docs and smoke assumptions for validation state naming"
    completed: true
  - id: task-32-4
    title: "Run pi-tandem smoke checks"
    completed: true
accord:
  status: "accepted"
  assignee: "review-validation-flow"
  claimedAt: "2026-06-28T15:16:45Z"
  deliveredAt: "2026-06-28T15:16:45Z"
  deliverables: ["code:extensions/pi-tandem/index.ts:prompt/tool guidance reflects Validation workflow if needed", "docs:extensions/pi-tandem/README.md:usage guidance updated", "docs:extensions/pi-tandem/plan/spec.md:adapter spec updated", "tests:extensions/pi-tandem:smoke checks updated and run"]
  validation:
    commands: ["bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts", "bun extensions/pi-tandem/tests/smoke.ts"]
  constraints: ["Keep pi-tandem a thin adapter over tandem; do not duplicate protocol parsing or mutation behavior.", "Do not promote to global Pi config in this task."]
  summary: "pi-tandem guidance and smoke assumptions now direct agents to deliver into Validation and avoid human-required acceptance/completion without approval."
  evidence: ["bun --check, smoke, relationship-smoke, and pi-runtime-smoke passed with TANDEM_BIN=tandem/target/debug/tandem."]
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/tests/relationship-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.md"]
  reviewer: "ivan"
  note: "Accepted as completed foundation for the Validation direction; remaining board/TUI cleanup is tracked separately."
  updatedAt: "2026-06-28T15:31:34Z"
---

## Description

Update pi-tandem prompt guidance, docs, and smoke assumptions for the Validation workflow.

Agents should deliver work into Validation, avoid completing human-required validation without approval, and keep using `tandem` tools rather than direct file edits.
