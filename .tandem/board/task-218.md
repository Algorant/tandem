---
id: task-218
type: task
title: "Build the read-only Tandem browser interface"
state: todo
priority: "high"
parentId: "task-216"
blockers: ["task-217"]
references: ["task-121"]
relatedFiles: ["tandem/src", "tandem/README.md"]
tags: ["ui", "web", "frontend", "accessibility"]
createdAt: "2026-08-05T18:46:16Z"
updatedAt: "2026-08-05T18:46:16Z"
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
