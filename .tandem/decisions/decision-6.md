---
id: decision-6
type: decision
title: "Define canonical protocol, project, app, and peer CLI/TUI architecture"
status: "accepted"
deciders: ["Algorant"]
tags: ["architecture", "protocol", "rust", "cli", "tui", "refactor"]
createdAt: "2026-08-31T20:52:47Z"
decidedAt: "2026-08-31T00:00:00Z"
updatedAt: "2026-08-31T20:52:47Z"
---

## Status

Accepted. Still in force for protocol 0.3.0.

## Decision

Adopt the ownership and dependency model: normative `protocol/` Markdown with `tandem/src/protocol/` as its executable implementation; `project::TandemProject` owns concrete project-local filesystem behavior; `app` composes shared typed use cases; CLI and TUI remain peer interfaces over protocol and app; `main.rs` remains process composition and exit wiring.

```text
protocol <- project
protocol + project <- app
protocol + app <- cli
protocol + app <- tui
cli + tui startup <- main
```

The TUI module root is `tui/mod.rs`; leaf files are created only when cohesive implementation moves into them.

## Consequences

- `protocol` owns meaning, `project` owns files, `app` owns shared operations, and peer `cli`/`tui` interfaces consume them.
- The repository stays one Cargo package and one production binary crate; no root workspace, lib.rs, or generic storage abstractions.
- Protocol/product changes stay separate from move-only extraction commits.
