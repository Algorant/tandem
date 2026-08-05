---
id: task-220
type: task
title: "Validate and document the Tandem web MVP"
state: "in-progress"
priority: "medium"
parentId: "task-216"
blockers: ["task-217", "task-218", "task-219"]
references: ["task-121"]
relatedFiles: ["tandem/README.md", "tandem/RELEASE.md", "docs", "RELEASES.md"]
tags: ["docs", "web", "validation", "packaging"]
createdAt: "2026-08-05T18:46:31Z"
updatedAt: "2026-08-05T19:24:01Z"
accord:
  status: "claimed"
  assignee: "worker-task-220-00a81d60"
  claimedAt: "2026-08-05T19:24:01Z"
  updatedAt: "2026-08-05T19:24:01Z"
assignee: "worker-task-220-00a81d60"
---

## Description

Complete cross-interface validation, packaging checks, and concise user documentation for the read-only web mode.

Acceptance criteria:
- Document startup, default browser behavior, `--port`, `--no-open`, read-only scope, loopback boundary, and deferred capabilities.
- Verify all web views against representative workspace data and canonical CLI/TUI meaning.
- Run Rust formatting, full tests, strict Clippy, docs build/link checks, and release-build/version checks.
- Verify bundled assets work from the packaged binary without the source tree or Node runtime.
- Perform desktop, narrow-screen, keyboard-only, Default Dark, and Verdigris browser smoke checks.
- Record remaining mutation, remote-access, SSE, database, and agent-feedback work as deferred rather than silently expanding this Epic.
