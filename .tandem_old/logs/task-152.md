---
id: task-152
type: task
title: "Extract executable protocol document and configuration semantics"
priority: "high"
parentId: "task-146"
blockers: ["task-151"]
references: ["decision-7"]
relatedFiles: ["plan/refactor_spec.md", "protocol/plan/spec.md", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/protocol/"]
tags: ["protocol", "rust", "architecture", "refactor"]
createdAt: "2026-07-22T20:41:09Z"
updatedAt: "2026-07-26T22:05:09Z"
accord:
  status: "accepted"
  assignee: "worker-task-152-f340e0c9"
  claimedAt: "2026-07-26T21:55:10Z"
  deliveredAt: "2026-07-26T22:04:57Z"
  deliverables: ["tandem/src/protocol/mod.rs normative/executable boundary docs", "protocol Document values/accessors, supported vocabularies, aliases, preservation semantics", "protocol config/version/workflow-state semantics", "root StoredDocument wrapper retaining concrete path/location ownership", "moved and added focused protocol tests"]
  validation:
    commands: ["cargo fmt --check", "174 unit + 4 executable tests passed", "strict Clippy passed", "real --version and --help checks passed", "protocol modules contain no filesystem/path/project/app/CLI/TUI dependency", "no protocol lint suppression", "worker clean"]
  summary: "Extracted executable protocol document and configuration semantics into protocol/document.rs and protocol/config.rs with a temporary root StoredDocument source wrapper."
  evidence: ["commit f90584b fast-forward integrated", "174 unit + 4 executable tests passed", "strict Clippy and formatting passed", "protocol dependency/import audit passed", "normative spec links present", "no new suppressions"]
  filesChanged: ["tandem/src/protocol/mod.rs", "tandem/src/protocol/document.rs", "tandem/src/protocol/config.rs", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/review.rs"]
  reviewer: "parent-orchestrator"
  note: "Reviewed the extraction and independently reran all gates. Protocol modules own logical values and configuration semantics while concrete source path/location and file operations remain in the temporary root wrapper. Delegating compatibility free functions only forward to canonical Document methods; no duplicate semantic implementation was found."
  updatedAt: "2026-07-26T22:05:03Z"
assignee: "worker-task-152-f340e0c9"
completedAt: "2026-07-26T22:05:09Z"
completion:
  summary: "Created the executable protocol boundary for logical document/configuration semantics, fixed vocabularies, supported types, workflow configuration, aliases, and unknown-field/body preservation while retaining concrete source ownership in a temporary root StoredDocument wrapper."
  filesChanged: ["tandem/src/protocol/mod.rs", "tandem/src/protocol/document.rs", "tandem/src/protocol/config.rs", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/review.rs"]
  validation: "Parent reviewed commit f90584b and independently passed formatting, 174 unit tests, 4 real-command tests, strict Clippy, real help/version checks, dependency/import audit, and suppression audit. Protocol modules have no concrete filesystem or reverse interface dependency."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Create the first executable-protocol module boundary by moving document/configuration meaning out of the binary root without changing approved protocol 0.2 behavior.

## Scope

- Establish `tandem/src/protocol/mod.rs`, `protocol/document.rs`, and `protocol/config.rs` only when cohesive implementation moves into them.
- Move logical Document values/accessors, required identity/type fields, configuration values, workflow-state configuration, fixed priority/effort vocabulary, supported-type handling, and unknown-field/body preservation requirements.
- Keep concrete discovery, path access, raw byte patching, locking, and writes out of protocol.
- Define narrow diagnostics/value APIs needed by later hierarchy, project, app, CLI, and TUI work without creating a public library API.
- Move focused tests with the code and replace wildcard imports with explicit visibility where touched.

## Acceptance criteria

- Normative `protocol/` Markdown remains the source of truth and module documentation links to it.
- Protocol code performs no concrete filesystem access and imports no project/app/CLI/TUI code.
- Approved protocol 0.2 behavior and exact diagnostics/output remain unchanged from task-150.
- No duplicate document/config interpretation remains in moved call sites.
- Formatting, full tests, real-command tests, strict Clippy, and dependency/visibility review pass.
- Temporary lint expectations assigned to this seam are removed.
- No hierarchy, project-I/O, application, interface redesign, release, or push occurs.

Creating this Task does not authorize starting it.
