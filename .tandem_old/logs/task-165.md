---
id: task-165
type: task
title: "Add safe active-Task Markdown body editing"
priority: "high"
references: ["task-107", "task-74", "task-146"]
relatedFiles: ["tandem/src/main.rs", "tandem/plan/spec.md", "protocol/plan/spec.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/README.md"]
tags: ["cli", "protocol", "pi-tandem", "bugfix", "rust"]
createdAt: "2026-07-23T01:20:30Z"
updatedAt: "2026-07-23T01:26:49Z"
accord:
  status: "accepted"
  assignee: "parent-agent"
  claimedAt: "2026-07-23T01:21:18Z"
  deliveredAt: "2026-07-23T01:26:28Z"
  deliverables: ["CLI `tandem update <id> --body <markdown>` accepts empty, whitespace, Unicode, multiline, and leading-dash bodies.", "pi-tandem `body` mapping preserves exact values, including empty strings, while `description` remains add-only.", "Protocol/CLI/extension docs and focused Rust/Bun coverage align."]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml --check passed", "cargo test --manifest-path tandem/Cargo.toml passed: 156 tests", "bun extensions/pi-tandem/tests/smoke.ts passed against the current repository binary", "bun extensions/pi-tandem/tests/relationship-smoke.ts passed", "cargo clippy --manifest-path tandem/Cargo.toml --all-targets completed with only the documented pre-existing 27/28 warnings and no warning in changed body-edit code", "git diff --check passed; focused commit 7a531a4; clean working tree"]
  summary: "Accepted after direct diff review and successful Rust/pi-tandem regression validation; implementation meets the agreed exact-body, preservation, no-op, privacy, and thin-adapter contract."
  evidence: ["Current-source process smoke previously confirmed the old --body rejection; repository smoke now round-trips exact and empty bodies through CLI/pi-tandem.", "Body-update event assertions prove no body content leaks into task.updated summaries and no-op writes/events are suppressed."]
  filesChanged: ["tandem/src/main.rs", "tandem/plan/spec.md", "protocol/plan/spec.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/plan/spec.md", "docs/cli/index.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-23T01:26:36Z"
completedAt: "2026-07-23T01:26:49Z"
completion:
  summary: "Added safe exact Markdown body editing for active Tasks across Tandem CLI and pi-tandem in commit 7a531a4."
  filesChanged: ["tandem/src/main.rs", "tandem/plan/spec.md", "protocol/plan/spec.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/plan/spec.md", "docs/cli/index.md"]
  validation: "156 Rust tests, pi-tandem smoke and relationship smoke, formatting, diff checks, and non-strict Clippy passed; Clippy emitted only documented pre-existing warnings outside changed code."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Add the missing first-class mutation path for replacing or clearing an active Task's complete Markdown body without direct `.tandem` file edits.

## Required behavior

- Support `tandem update <id> --body <markdown>` for active task documents only.
- Treat the value as the exact body text after the closing frontmatter delimiter, including empty/whitespace-only content, leading `-`, Unicode, and leading/trailing newlines.
- Preserve all unrelated raw frontmatter and unknown fields; change only the body and `updatedAt` when the body differs.
- A byte-identical body is a true no-op: no write, timestamp change, or event.
- Emit `task.updated` only for a real change and never include old/new body contents in human output or event summaries.
- Expose presence-sensitive pi-tandem `body` mapping for `action=update`; retain `description` as add-only behavior and keep accord/review on their dedicated flows.

## Acceptance criteria

- Unit and real-command coverage proves replacement, clearing, whitespace, leading-dash, Unicode/newline round trips, unknown-field/frontmatter preservation, no-op behavior, conflict safety, and event creation/suppression.
- pi-tandem mapping tests cover empty and flag-looking body values without silently dropping them.
- CLI help, protocol-facing command documentation, Tandem spec, extension schema/guidance, and tests agree.
- Existing metadata update, hierarchy, completion, TUI editor, and Decision behavior remain unchanged.
- Formatting, full Rust tests, focused pi-tandem Bun smokes, strict relevant checks, and diff checks pass.
- Deliver one focused implementation commit on `main`; do not release, push, begin Epic task-146, or modify `/usr/bin/tandem` in this Task.
