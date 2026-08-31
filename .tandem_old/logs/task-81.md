---
id: task-81
type: task
kind: "epic"
title: "Strengthen Decision docs with ADR-compatible structure"
priority: "high"
relatedFiles: ["protocol/plan/spec.md", "tandem/plan/spec.md", "extensions/pi-tandem/plan/spec.md", "README.md"]
tags: ["protocol", "decision", "adr", "cli", "tui", "docs", "extensions"]
createdAt: "2026-07-01T17:26:25Z"
updatedAt: "2026-07-04T11:36:34Z"
subtasks:
  - id: task-81-1
    title: "Review Decision required and optional metadata against common ADR expectations"
    completed: false
  - id: task-81-2
    title: "Define recommended Decision/ADR body template"
    completed: false
  - id: task-81-3
    title: "Fix decision rendering/listing so workflow state is not required"
    completed: false
  - id: task-81-4
    title: "Update CLI/TUI/docs/extensions guidance while preserving canonical Decision terminology"
    completed: false
completedAt: "2026-07-04T11:36:34Z"
completion:
  summary: "Completed parent ADR-compatible Decision support task after validating that all child protocol/CLI, TUI, documentation, and agent guidance work has been completed and accepted."
  validation: "User requested marking task-81 validated/complete. Child work completed: task-85 Decision ADR protocol/CLI core, task-86 Decision TUI rendering and board classification with visual approval, and task-87 ADR-compatible Decision documentation/agent guidance."
---

## Description

Keep `decision` as the canonical Tandem term while making Decision documents more systematic and compatible with common ADR expectations.

Agreed direction:
- Keep the first-class document type, CLI namespace, TUI pane, and extension terminology as `decision` / `Decisions`.
- Do not add a separate `adr` document type or rename the feature to ADR.
- Under the hood and in documentation, support ADR-compatible conventions for durable architecture/product/project decisions.
- Decisions should not use normal board workflow `state`; decision-specific status/metadata should be separate if needed.
- Fix or clarify rendering so decisions without workflow state do not appear as `unfiled` board items.

Expected changes:
- Protocol/spec: review required and optional Decision fields, including ADR-typical metadata such as status, date, deciders, context, consequences, alternatives, supersedes/superseded-by, and references.
- Docs: define a recommended Decision/ADR body template while keeping the canonical name Decision.
- CLI: streamline decision add/show/list output and metadata handling as needed.
- TUI: ensure Decisions pane renders decision metadata/body cleanly and does not depend on task workflow state.
- Extensions/skills: update guidance so agents use Tandem Decisions in an ADR-compatible way without inventing separate ADR protocol behavior.
