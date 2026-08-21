---
id: task-235
type: task
title: "Restore fast Tandem web reference validation"
state: "in-progress"
priority: "high"
effort: "small"
references: ["task-231"]
relatedFiles: ["tandem/src/app/queries.rs", "tandem/src/project/mod.rs", "tandem/src/web.rs"]
tags: ["ui", "web", "performance", "papercuts"]
createdAt: "2026-08-21T21:18:31Z"
updatedAt: "2026-08-21T21:19:47Z"
accord:
  status: "claimed"
  assignee: "worker-task-235-8bc3ab28"
  claimedAt: "2026-08-21T21:19:47Z"
  updatedAt: "2026-08-21T21:19:47Z"
assignee: "worker-task-235-8bc3ab28"
---

## Description

Fix the Papercut reference regression diagnosed in task-231. In `app::queries::load_read`, resolve Task, Decision, and Log references through the already-loaded `ProjectHierarchy` instead of calling `TandemProject::reference_target_exists` for every reference. Add a narrow project-layer helper that recognizes a canonical Papercut ID and checks only `.tandem/papercuts/<id>.md`. Preserve missing-reference warnings and the rule that an existing canonical Papercut filename counts as a target without parsing its contents, so malformed Papercuts cannot break unrelated web or Board reads.

Keep this as a direct performance correction. Do not add caching, snapshot coordination, new HTTP endpoints, locking changes, SSE/WebSockets, remote access, or server architecture.

Acceptance criteria:
1. `load_read` does not rescan Board or Log documents per loose reference.
2. References to active documents, completed Logs, and existing canonical Papercut filenames resolve correctly.
3. Missing references still warn.
4. Malformed Papercut contents do not make `load_read` fail solely because the filename is referenced.
5. Focused regression tests pass.
6. `/api/v1/project` loads successfully on the Tandem and `~/.pi` workspaces, with before/after timing recorded.
