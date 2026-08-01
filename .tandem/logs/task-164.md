---
id: task-164
type: task
title: "Validate, synchronize, and merge the completed architecture refactor"
priority: "high"
parentId: "task-146"
blockers: ["task-163"]
references: ["task-145", "decision-7"]
relatedFiles: ["plan/refactor_spec.md", "tandem/", "protocol/", "extensions/pi-tandem/", "AGENTS.md", "README.md"]
tags: ["rust", "architecture", "testing", "refactor"]
createdAt: "2026-07-22T20:43:33Z"
updatedAt: "2026-07-29T00:51:12Z"
accord:
  status: "accepted"
  assignee: "worker-task-164-6ee31847"
  claimedAt: "2026-07-29T00:44:44Z"
  deliveredAt: "2026-07-29T00:50:28Z"
  deliverables: ["Commit 0fa4955", "Campaign branch refactor/protocol-architecture validated at 34 commits ahead and 0 behind main", "Final campaign-to-main merge readiness evidence"]
  validation:
    commands: ["206 unit + 6 real-command tests passed", "Strict Clippy zero diagnostics and formatting passed", "Release build/version and one-package metadata passed", "Protocol/project/app dependency and terminology audits passed", "Pi extension syntax and all three smokes passed", "Astro/Bun docs build and 605-link check passed", "Release-check Python tests passed", "PTY smoke passed", "Direct final just dev inspection passed for State/Epic Board, Logs, Rules, Decisions, and Help", "git diff --check and clean status passed"]
  constraints: ["No release or push performed"]
  summary: "Accepted after final audit review, independent direct just dev approval of all top-level views, Worker integration, and exact-history fast-forward of 34 campaign commits into local main."
  evidence: ["main was 0 ahead / campaign 33 ahead before final audit fix", "Only retained Review suppressions remain and are documented product-decision code", "Preview route reset after direct validation"]
  filesChanged: ["tandem/src/app/support.rs"]
  reviewer: "orchestrator"
  updatedAt: "2026-07-29T00:51:07Z"
assignee: "worker-task-164-6ee31847"
completedAt: "2026-07-29T00:51:12Z"
completion:
  summary: "Completed final architecture audit and merged refactor/protocol-architecture into local main with exact 34-commit history at 0fa4955."
  filesChanged: ["tandem/src/app/support.rs"]
  validation: "All Rust, real-command, strict Clippy, release-build/version, extension smoke, Astro/link, packaging-check, PTY, terminology/dependency, and direct just dev validations passed."
  reviewer: "orchestrator"
---

## Description

## Objective

Perform the final campaign-wide architecture review and merge the completed integration branch to `main` only when every automated, structural, documentation, and human-visible requirement is satisfied.

## Scope

- Synchronize eligible final `main` changes into `refactor/protocol-architecture` and resolve drift without weakening boundaries.
- Review the complete branch against the accepted architecture decision, `plan/refactor_spec.md`, every direct Task delivery, and normative protocol 0.2 documents.
- Remove every temporary refactor lint expectation/allowance and migration-only visibility/import workaround.
- Confirm one canonical implementation for document semantics, IDs, hierarchy, workflow, accord, review, events, and diagnostics.
- Confirm `project::TandemProject` owns concrete `.tandem` I/O and CLI/TUI use shared app mutations.
- Run final Rust, CLI, project-file, hierarchy/concurrency, TUI/PTTY, documentation, packaging-relevant, and human `just dev` validation.
- Merge the reviewed integration branch to `main` only after parent/human acceptance.

## Acceptance criteria

- Formatting, all tests, real-command behavior tests, strict `cargo clippy --all-targets -- -D warnings`, terminology/import checks, and documentation validation pass with no unexplained test loss.
- Strict Clippy reports zero diagnostics and no temporary refactor suppression remains.
- `main.rs` and `tui/mod.rs` are wiring roots; dependency direction and visibility match the decision.
- CLI/TUI outputs and visible behavior are approved, including genuine human `just dev` validation of all top-level TUI views.
- The integration branch is clean, attributable, synchronized, and merged to `main` without unreviewed changes.
- Epic task-146 is ready for owner acceptance/completion after this Task; this Task does not itself publish a release or push unless separately requested.

Creating this Task does not authorize starting it.
