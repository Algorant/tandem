---
id: task-218
type: task
title: "Build the read-only Tandem browser interface"
priority: "high"
parentId: "task-216"
blockers: ["task-217"]
references: ["task-121"]
relatedFiles: ["tandem/src", "tandem/README.md"]
tags: ["ui", "web", "frontend", "accessibility"]
createdAt: "2026-08-05T18:46:16Z"
updatedAt: "2026-08-05T19:13:27Z"
accord:
  status: "accepted"
  assignee: "worker-task-218-c8fd3a7d"
  claimedAt: "2026-08-05T19:02:57Z"
  deliveredAt: "2026-08-05T19:13:12Z"
  deliverables: ["Responsive semantic browser UI for every MVP read view.", "Vanilla JavaScript API, routing, rendering, filtering, search, selection, loading, empty, and error states.", "Safe small-subset Markdown rendering and documented `tandem web` usage."]
  validation:
    commands: ["Post-integration formatting passed.", "Post-integration 237 unit and 11 integration tests passed.", "Post-integration strict all-feature Clippy and release build passed.", "Node syntax checks passed for app.js, api.js, and ui.js.", "Parent Chromium smoke captured and reviewed wide and narrow responsive Board views; Worker smoke covered all required views and detail flows."]
  summary: "Built the bundled read-only Tandem browser interface across Board, Validation, details, Logs, Rules, Decisions, and project health with responsive Verdigris styling and accessible interaction."
  evidence: ["Integrated commit 3e0a? pending actual HEAD verified through Worktrunk merge; source commit 926119d.", "Reviewed wide and 390px narrow screenshots for hierarchy, navigation, filters, status labels, focus treatment, and responsive layout.", "UI remains bundled, read-only, same-origin, framework-free, and dependent on API DTOs rather than Tandem Markdown parsing."]
  filesChanged: ["tandem/src/web.rs", "tandem/src/web/index.html", "tandem/src/web/app.css", "tandem/src/web/app.js", "tandem/src/web/api.js", "tandem/src/web/ui.js", "tandem/README.md"]
  reviewer: "orchestrator"
  note: "Accepted under the user's explicit instruction that the orchestrator evaluate Worker results. Reviewed integrated commit aad2d0e, all required views, wide and narrow Chromium screenshots, accessible focus/navigation treatment, and full validation. Refresh/security hardening remains correctly scoped to task-219."
  updatedAt: "2026-08-05T19:13:22Z"
assignee: "worker-task-218-c8fd3a7d"
completedAt: "2026-08-05T19:13:27Z"
completion:
  summary: "Built and visually validated the bundled read-only browser interface for Board, Validation, details, Logs, Rules, Decisions, and project health with responsive Verdigris styling and accessible vanilla JavaScript interactions."
  filesChanged: ["tandem/src/web.rs", "tandem/src/web/index.html", "tandem/src/web/app.css", "tandem/src/web/app.js", "tandem/src/web/api.js", "tandem/src/web/ui.js", "tandem/README.md"]
  validation: "Integrated commit aad2d0e reviewed; 237 unit and 11 integration tests, formatting, strict all-feature Clippy, release build, JS syntax, API smoke, all-view Chromium smoke, and wide/narrow visual review passed."
  reviewer: "orchestrator"
---

## Description

Build a bundled semantic HTML interface with small vanilla JavaScript modules over the `/api/v1` read API.

Acceptance criteria:
- Include Board and Validation, document details and relationships, Logs, Rules, Decisions, and project health.
- Preserve canonical hierarchy, status, warning, and completion meaning from API view models.
- Use responsive Verdigris styling and existing semantic theme concepts.
- Support keyboard navigation, visible focus, narrow screens, zoom, and status text that does not depend on color alone.
- Keep transient filters and selection in the browser only; do not duplicate protocol logic.
- Add focused rendering and browser smoke coverage.
