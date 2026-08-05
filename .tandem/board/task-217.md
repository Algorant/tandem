---
id: task-217
type: task
title: "Add the embedded web server and read API"
state: todo
priority: "high"
parentId: "task-216"
references: ["task-121"]
relatedFiles: ["tandem/Cargo.toml", "tandem/src/main.rs", "tandem/src/app", "tandem/src/project", "tandem/src/protocol"]
tags: ["ui", "web", "server", "api"]
createdAt: "2026-08-05T18:46:08Z"
updatedAt: "2026-08-05T18:46:08Z"
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
