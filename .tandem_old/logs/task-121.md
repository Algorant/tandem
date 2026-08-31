---
id: task-121
type: task
title: "Research a lightweight Tandem web server and UI"
priority: "medium"
references: ["task-120"]
relatedFiles: ["tandem/src", "tandem/plan/spec.md"]
tags: ["ui", "web", "server", "research"]
createdAt: "2026-07-10T12:44:41Z"
updatedAt: "2026-07-26T14:47:20Z"
accord:
  status: "accepted"
  assignee: "worker-task-121-8ae7e8b8"
  claimedAt: "2026-07-26T14:27:04Z"
  deliveredAt: "2026-07-26T14:47:04Z"
  deliverables: ["plan/web-ui-research.md"]
  validation:
    commands: ["git diff HEAD~2..HEAD --check"]
  summary: "Delivered lightweight Tandem web server/UI architecture research in plan/web-ui-research.md."
  evidence: ["Integrated commit be8ddb7"]
  filesChanged: ["plan/web-ui-research.md"]
  reviewer: "orchestrator"
  note: "Research deliverable covers requested architecture options, MVP, API/components, update strategy, security, parity roadmap, agent interaction, risks, and open questions. Acceptance is of the research deliverable, not the proposed architecture as a settled decision."
  updatedAt: "2026-07-26T14:47:15Z"
assignee: "worker-task-121-8ae7e8b8"
completedAt: "2026-07-26T14:47:20Z"
completion:
  summary: "Completed lightweight Tandem web server/UI research; architecture remains a research proposal pending product/security decisions."
  filesChanged: ["plan/web-ui-research.md"]
  validation: "Reviewed integrated research document; git diff check passed."
  reviewer: "orchestrator"
---

## Description

Research what it would take to provide a lightweight browser-based Tandem server/UI that initially displays the information currently available in the TUI, with a path toward broader TUI feature parity.

Explore an incremental scope:
1. A lightweight, primarily read-only web view covering the current Board, Validation/Review, Logs, Rules, Decisions, task details, relationships, accord status, and relevant project metadata.
2. Real-time or refresh-based updates as workspace state changes.
3. Later mutation support, including creating and updating tasks and other TUI-equivalent actions.
4. A future interaction channel for users to send feedback or review decisions back to agents safely from the web UI.

Evaluate:
- an embedded server in the existing Rust application versus a separate service/frontend;
- how to reuse Tandem's protocol and mutation logic without duplicating file parsing or business rules;
- API boundaries, event/update transport, frontend complexity, packaging, startup, and local deployment;
- localhost-first defaults, remote access, authentication, authorization, CSRF, secrets, and workspace isolation;
- multi-user and concurrent mutation behavior;
- agent identity, feedback routing, task creation, validation/acceptance flows, and auditability;
- how database/sync-provider work could support remote or persistent deployments without becoming a prerequisite for a local MVP;
- accessibility, responsive layout, theming, and a practical feature-parity roadmap.

Deliverable: a concise architecture/options analysis, recommended lightweight MVP, proposed API and component boundaries, security model, staged roadmap toward TUI parity and agent interaction, risks, and open questions. Keep this as research until the approach is reviewed.
