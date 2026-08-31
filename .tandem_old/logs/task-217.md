---
id: task-217
type: task
title: "Add the embedded web server and read API"
priority: "high"
parentId: "task-216"
references: ["task-121"]
relatedFiles: ["tandem/Cargo.toml", "tandem/src/main.rs", "tandem/src/app", "tandem/src/project", "tandem/src/protocol"]
tags: ["ui", "web", "server", "api"]
createdAt: "2026-08-05T18:46:08Z"
updatedAt: "2026-08-05T19:02:28Z"
accord:
  status: "accepted"
  assignee: "worker-task-217-58f38cf0"
  claimedAt: "2026-08-05T18:49:34Z"
  deliveredAt: "2026-08-05T19:02:20Z"
  deliverables: ["`tandem web [--port <port>] [--no-open]` with loopback binding and browser startup behavior.", "Read-only APIs for project health, Board/Validation, document relationships, Logs, Rules, and Decisions.", "Bundled shell assets and explicit safe error/envelope responses."]
  validation:
    commands: ["Post-integration cargo fmt check passed.", "Post-integration 235 unit and 11 integration tests passed.", "Post-integration strict Clippy and release build passed.", "Live release-binary smoke served the shell, project envelope, Board data, revision, and warnings from 127.0.0.1 and stopped cleanly."]
  summary: "Added the embedded loopback-only Tandem web server, startup command, bundled shell assets, opaque revisions, and read-only `/api/v1` DTOs over shared project/app queries."
  evidence: ["Integrated commit bd983fc into main via Worktrunk.", "Reviewed route set, DTO boundary, app/project query ownership, CLI parsing, dependencies, and safe error behavior.", "No mutation, arbitrary workspace selection, remote binding, database, SSE, or WebSocket routes were added."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/src/app/queries.rs", "tandem/src/cli/args.rs", "tandem/src/cli/landing.rs", "tandem/src/cli/mod.rs", "tandem/src/main.rs", "tandem/src/project/mod.rs", "tandem/src/web.rs", "tandem/src/web/app.css", "tandem/src/web/app.js", "tandem/src/web/index.html", "tandem/tests/cli_behavior.rs"]
  reviewer: "orchestrator"
  note: "Accepted after integrated code review, full Rust validation, strict Clippy, release build, and live loopback API smoke. Visual interface work remains correctly scoped to task-218."
  updatedAt: "2026-08-05T19:02:23Z"
assignee: "worker-task-217-58f38cf0"
completedAt: "2026-08-05T19:02:28Z"
completion:
  summary: "Added the embedded loopback-only Tandem web server, browser-startup command, bundled shell, opaque revisions, and canonical read-only APIs for all MVP data views."
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/src/app/queries.rs", "tandem/src/cli/args.rs", "tandem/src/cli/landing.rs", "tandem/src/cli/mod.rs", "tandem/src/main.rs", "tandem/src/project/mod.rs", "tandem/src/web.rs", "tandem/src/web/app.css", "tandem/src/web/app.js", "tandem/src/web/index.html", "tandem/tests/cli_behavior.rs"]
  validation: "Integrated review passed; 235 unit and 11 integration tests passed; strict Clippy, formatting, release build, diff check, and live release-binary shell/API smoke passed."
  reviewer: "orchestrator"
---

## Description

Implement `tandem web [--port <port>] [--no-open]` in the existing Rust binary. Add an embedded Axum server, bundled static assets, one pinned workspace per process, startup/browser behavior, explicit `/api/v1` DTOs, and canonical read queries through shared app/project boundaries.

Acceptance criteria:
- Loopback-only binding with available-port selection and clear startup output.
- Browser opens by default and `--no-open` suppresses it.
- Read APIs cover project health, Board/Validation, document detail and relationships, Logs, Rules, and Decisions.
- Responses include an opaque project revision and warnings.
- No raw Tandem files, arbitrary filesystem paths, duplicated parsing, or mutation routes.
- Focused API, startup, error-mapping, and boundary tests pass.
