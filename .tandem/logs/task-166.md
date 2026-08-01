---
id: task-166
type: task
title: "Add auditable active-Task cancellation to Logs"
priority: "high"
blockers: ["task-165"]
references: ["task-107", "task-146", "decision-6"]
relatedFiles: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/logs.rs", "tandem/plan/spec.md", "protocol/plan/spec.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts"]
tags: ["cli", "protocol", "tui", "pi-tandem", "rust"]
createdAt: "2026-07-23T01:20:49Z"
updatedAt: "2026-07-23T01:51:45Z"
accord:
  status: "accepted"
  assignee: "parent-agent"
  claimedAt: "2026-07-23T01:27:05Z"
  deliveredAt: "2026-07-23T01:51:16Z"
  deliverables: ["Rust CLI cancellation with hierarchy lock, descendant rejection, compatible completion.outcome metadata, duplicate protection, ID continuity, and event append.", "Outcome-aware human/JSON show/search/log output, TUI Logs rendering/validation, and successful-completion rollup exclusion.", "pi-tandem action=cancel/reason mapping, authority guidance, smoke coverage, protocol/spec/README/public docs alignment."]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml --check passed", "cargo test --manifest-path tandem/Cargo.toml passed: 161 tests", "Bun syntax checks passed for pi-tandem extension and three smokes", "pi-tandem smoke, relationship smoke, and project-local Pi runtime smoke passed", "Current-source CLI fixture verified active-descendant rejection, cancel archive/event, JSON outcome, body preservation, search label, and ID non-reuse", "cargo clippy --all-targets exited 0 with only the documented pre-existing 27/28 warnings and no cancellation/body-edit warning", "Docs site built 15 pages; 605 internal links passed; high-level Bun audit clean", "Genuine `just dev` validation approved Board rollup (1 active · 1 logged excluding canceled), completed/canceled Logs rows, CANCELED badge, cancellation detail/body/event timeline, canceled header/status, keyboard selection, and cleanup", "git diff --check passed; focused commit e88a85f; clean working tree"]
  summary: "Accepted after direct diff review, automated protocol/CLI/pi-tandem/TUI validation, and genuine human `just dev` inspection. The implementation meets the agreed cancellation MVP without permanent deletion or architecture work."
  evidence: ["Disposable `/tmp/tandem-v061-cancel-preview` fixture and Herdr tab showed one completed and one canceled Log; route/tab/fixture were reset and removed afterward.", "Canceled blocker resolution, active-descendant rejection, duplicate ID/destination rejection, legacy completed default, unknown outcome diagnostics, and progress exclusion have focused unit coverage."]
  filesChanged: ["AGENTS.md", "README.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/README.md", "tandem/RELEASE.md", "tandem/plan/spec.md", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/logs.rs", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/plan/spec.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/protocol/index.md", "docs/quick-start/index.md", "docs/tui/index.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-23T01:51:28Z"
completedAt: "2026-07-23T01:51:45Z"
completion:
  summary: "Added auditable active-Task cancellation with compatible canceled Logs, outcome-aware CLI/JSON/TUI behavior, progress exclusion, and pi-tandem support in commit e88a85f."
  filesChanged: ["AGENTS.md", "README.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/README.md", "tandem/RELEASE.md", "tandem/plan/spec.md", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/logs.rs", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/plan/spec.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/protocol/index.md", "docs/quick-start/index.md", "docs/tui/index.md"]
  validation: "161 Rust tests; three pi-tandem/Bun smokes; syntax, formatting, diff, Clippy baseline, docs build, 605 links, high audit, real-command fixture, and genuine just dev human validation passed."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Provide a safe way to cancel mistaken, abandoned, or intentionally discontinued active Tasks while retaining audit history and preventing ID reuse, without misrepresenting cancellation as successful completion or permanent deletion.

## Required behavior

- Support `tandem cancel <id> --reason <text>` for active task documents only.
- Reject cancellation when the target has any active descendant; do not cascade.
- Permit cancellation regardless of blockers, review, or accord acceptance.
- Preserve body/frontmatter/unknown fields, remove active `state`, update `updatedAt`, set `completedAt`, and write nested `completion.outcome: canceled` plus `completion.summary: "Canceled: <reason>"`.
- Reject an existing destination Log; retain the canonical ID so allocation never reuses it.
- Emit `task.canceled`; treat cancellation as terminal so it resolves blocker references.
- Default missing `completion.outcome` to `completed` for legacy Logs.
- Make human/JSON Log reads, show/search, TUI Logs, hierarchy context, diagnostics, and Board rollups label cancellation correctly and exclude canceled work from successful-completion progress.
- Expose pi-tandem `action=cancel` with required `reason`.

## Acceptance criteria

- Protocol/spec/help document the additive backward-compatible outcome field and cancellation command while retaining `protocolVersion: 0.1.0` for v0.6.1.
- Automated tests cover archive metadata/body preservation, no active state, event creation, duplicate destination, active-descendant rejection, blocker resolution, ID non-reuse, legacy completed default, CLI/JSON/TUI rendering, progress exclusion, and pi-tandem mapping.
- TUI has read/render compatibility but no cancel action in this MVP; human `just dev` validation confirms completed and canceled Logs/Board context remain clear.
- Formatting, full Rust tests, pi-tandem Bun smokes, strict relevant checks, and diff checks pass.
- Deliver one focused implementation commit on `main`; do not permanently delete records, cascade, recreate IDs, release, push, begin Epic task-146, or modify `/usr/bin/tandem` in this Task.
