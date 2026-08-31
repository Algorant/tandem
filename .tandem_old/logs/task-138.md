---
id: task-138
type: task
title: "Validate Git union event behavior and publish migration guidance"
priority: "high"
blockers: ["task-136", "task-137"]
references: ["decision-6", "decision-7"]
relatedFiles: ["docs", "README.md", "tandem/README.md", "tandem/RELEASE.md", "tandem/src", "justfile"]
tags: ["integration", "tests", "docs", "git", "cross-platform", "events"]
createdAt: "2026-07-15T19:44:05Z"
updatedAt: "2026-07-30T13:05:49Z"
parentId: "task-133"
completedAt: "2026-07-30T13:05:49Z"
completion:
  outcome: "canceled"
  summary: "Canceled: Superseded by decision-9. Union-merge integration and migration work is unnecessary because the project will retain the implemented per-actor event model."
---

## Description

This is a direct Task of Epic task-133. Integrate and validate the completed writer/reader work, then align user-facing documentation and release guidance with the shipped behavior.

Acceptance criteria:
- Add an end-to-end temporary-repository test that creates divergent branches, appends unique events, merges through the tracked union attribute, and verifies both events survive regardless of line order.
- Cover clone-local actor sharing across linked worktrees, independent actor IDs across clones, non-Git fallback, CRLF/LF normalization expectations, existing-workspace attribute augmentation, legacy mixed reads, and corruption diagnostics in integration validation.
- Update public protocol, concepts, quick-start, CLI/TUI, README, and release documentation so none claim per-actor files, external XDG storage, or `<actor>:<seq>` behavior.
- Document that the ledger is tracked, append-only, union-merged, semantically ordered, and not rotated initially.
- Explain existing workspace behavior and legacy event compatibility without requiring destructive migration or history rewriting.
- Run full Rust formatting/tests/build, Bun docs build/link checks, smoke tests, and diff/status validation.
- Confirm no platform-specific path assumptions and record any remaining genuinely deferred work as new Tasks rather than hidden TODOs.
