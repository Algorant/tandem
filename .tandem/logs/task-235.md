---
id: task-235
type: task
title: "Restore fast Tandem web reference validation"
priority: "high"
effort: "small"
references: ["task-231"]
relatedFiles: ["tandem/src/app/queries.rs", "tandem/src/project/mod.rs", "tandem/src/web.rs"]
tags: ["ui", "web", "performance", "papercuts"]
createdAt: "2026-08-21T21:18:31Z"
updatedAt: "2026-08-21T21:24:22Z"
accord:
  status: "accepted"
  assignee: "worker-task-235-8bc3ab28"
  claimedAt: "2026-08-21T21:19:47Z"
  deliveredAt: "2026-08-21T21:24:14Z"
  deliverables: ["In-memory active document and Log reference resolution in load_read", "Narrow canonical Papercut filename existence helper", "Regression coverage for active documents, Decisions, Logs, malformed Papercuts, and missing targets"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml -- --check", "cargo test --manifest-path tandem/Cargo.toml: 274 unit and 11 integration tests passed", "cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings", "Live /api/v1/project: Tandem 120 ms, /home/ivan/.pi 161 ms, HTTP 200 and valid JSON"]
  summary: "Restored fast reference validation by reusing the loaded ProjectHierarchy for document targets and checking only canonical Papercut filenames for Papercut targets."
  evidence: ["Commit d942da2", "Parent inspected the complete two-file diff after Worker delivery", "No caching, HTTP redesign, locking changes, or future server scope added"]
  filesChanged: ["tandem/src/app/queries.rs", "tandem/src/project/mod.rs"]
  reviewer: "pi-orchestrator"
  note: "Accepted after parent diff inspection, full Rust validation, strict Clippy, and live project-scale API timing on both Tandem and /home/ivan/.pi."
  updatedAt: "2026-08-21T21:24:17Z"
assignee: "worker-task-235-8bc3ab28"
completedAt: "2026-08-21T21:24:22Z"
completion:
  summary: "Restored fast Tandem web reference validation by reusing the loaded hierarchy for document targets and limiting Papercut checks to canonical filenames. Live API response time returned to 120 ms for Tandem and 161 ms for /home/ivan/.pi."
  filesChanged: ["tandem/src/app/queries.rs", "tandem/src/project/mod.rs"]
  validation: "Formatting passed; 274 unit and 11 integration tests passed; strict Clippy passed; /api/v1/project returned HTTP 200 with valid JSON in 120 ms on Tandem and 161 ms on /home/ivan/.pi."
  reviewer: "pi-orchestrator"
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
