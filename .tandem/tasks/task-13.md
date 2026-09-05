---
id: task-13
type: task
title: "Provide a complete current assignment contract for native coordination clients"
state: todo
priority: "high"
relatedFiles: ["tandem/src/app/dto.rs", "tandem/src/app/queries.rs", "tandem/src/cli/commands.rs", "tandem/src/project/mod.rs", "protocol/plan/spec.md"]
tags: ["protocol", "delegation"]
accord:
  status: "ready"
  acceptance: ["Verify version-matched source/native behavior and document the smallest complete assignment retrieval using existing capabilities or a narrowly justified native extension.", "Native/CLI tests retrieve complete root and milestone definition without truncation and distinguish actual scope edits from status/evidence/progress changes or unrelated Task changes.", "Dependency/readiness results identify missing native inputs with exact IDs/reasons using authoritative hierarchy and blockers; no new persisted readiness state or copied task store.", "Consumer handoff specifies exact reproducible command/JSON and version/commit availability for Pi; no unsupported API assumptions or intermediate-history system are introduced."]
  validation: ["Run relevant native/CLI fixture tests and cargo test --manifest-path tandem/Cargo.toml according to repository guidance; report exact scenario assertions and baseline issues separately."]
  constraints: ["Core Tandem scope only; no Pi adapter/config changes.", "No release push, installation or broad migration is implied by creating this proposal; report delivery availability explicitly.", "Do not add a new API merely to duplicate existing show operations; prove the gap first."]
  updatedAt: "2026-09-05T21:03:22Z"
createdAt: "2026-09-05T21:03:22Z"
updatedAt: "2026-09-05T21:03:22Z"
---

## Description

Implementation proposal from Pi task-59 at /home/ivan/.pi. Source research: /home/ivan/.pi/plan/pi-coordination-workflow-proposal.md. User wants simple, fast coordination, native CLI/TUI authority, Task=assignment/Subtask=milestone, and no attempt-history system. Begin by matching the source checkout to the relevant release: inspected checkout was tandem-v0.12.2-1-gc35090a while installed CLI was 0.12.3. Do not diagnose shipped behavior from the older checkout alone.

Outcome: Pi and other native clients can obtain the complete current assignment, including exact acceptance/constraints/planned validation, dependencies and milestone definitions, and compare current scope without treating ordinary progress as a scope change. Existing show already provides root detail and child summaries; use existing native operations where sufficient. Do not invent another task database, persisted ready/frozen state, definition-history archive, or generic graph API. Verify whether a narrow native definition token/projection is necessary; the current whole-workspace snapshot revision includes all document fields/progress and is unsuitable as-is. Keep taxonomy, parsing, readiness semantics and scope projection native.

Return a bounded consumer handoff to Pi task-59: exact CLI command/JSON, which existing capabilities suffice, any new fields/actions, behavioral tests and commit/release availability. Do not edit Pi adapters here or assume source completion means the installed CLI has changed. This task is not a prerequisite for the first Pi SDK/Worker proof unless that proof identifies a concrete native gap.
