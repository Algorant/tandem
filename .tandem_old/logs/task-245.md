---
id: task-245
type: task
title: "Stop cloning the complete hierarchy for indexed read queries"
priority: "high"
effort: "small"
references: ["task-141", "task-226", "task-233", "task-244", "decision-8"]
relatedFiles: ["tandem/src/project/mod.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/tui/logs.rs"]
tags: ["tui", "performance", "hierarchy"]
createdAt: "2026-08-29T04:09:30Z"
updatedAt: "2026-08-29T04:26:45Z"
accord:
  status: "accepted"
  assignee: "worker-task-245-e343b77d"
  claimedAt: "2026-08-29T04:14:11Z"
  deliveredAt: "2026-08-29T04:19:45Z"
  deliverables: ["Zero-copy canonical ProjectHierarchy query path using Cow", "Owned prospective replacement path with full identity/content check", "Structural tests for borrowed and owned paths"]
  validation:
    commands: ["cargo fmt --check passed", "cargo test passed: 284 unit tests and 12 integration tests", "cargo clippy --all-targets --all-features -- -D warnings passed", "cargo build --release passed", "Fixed 150x46 release PTY navigation redraw output followed each key in about 2-5 ms", "150-key 30 Hz held-j simulation drained continuously with no post-release backlog"]
  summary: "Canonical ProjectHierarchy queries now borrow the existing protocol hierarchy; modified/prospective documents still rebuild an isolated owned index."
  filesChanged: ["tandem/src/project/mod.rs"]
  note: "Automated validation, fixed-size PTY measurements, and user validation in the actual terminal all passed."
  updatedAt: "2026-08-29T04:26:40Z"
assignee: "worker-task-245-e343b77d"
review.note: "User validated the integrated release TUI in the actual terminal: Logs held-key navigation is much better."
review.requestedAt: "2026-08-29T04:19:50Z"
review.reviewer: "user"
review.status: "accepted"
review.decidedAt: "2026-08-29T04:26:35Z"
completedAt: "2026-08-29T04:26:45Z"
completion:
  summary: "Stopped canonical ProjectHierarchy queries from deep-cloning the complete protocol hierarchy. Existing indexed documents now borrow the coherent index; modified or prospective documents still rebuild an isolated owned index after full path, location, fields, and body comparison. Added structural tests for both paths. Validation passed: formatting, 284 unit tests, 12 integration tests, strict Clippy, release build, fixed 150x46 redraws at about 2-5 ms, a 150-key 30 Hz held-j simulation with no backlog, and user confirmation that actual-terminal Logs navigation is much better."
  filesChanged: ["tandem/src/project/mod.rs"]
  validation: "cargo fmt --check; cargo test (284 unit + 12 integration); strict Clippy; cargo build --release; fixed 150x46 PTY timing; held-key simulation; user actual-terminal validation"
  reviewer: "user"
---

## Description


## Description

Fix the fundamental hierarchy-query cost that makes ordinary Logs navigation take roughly 60 ms per redraw and causes held `j`/`k` input to backlog.

## Root cause

`ProjectHierarchy` already owns one coherent `ProtocolHierarchyIndex` for the loaded Board and Logs. Its `index_for` adapter (`tandem/src/project/mod.rs:352-365`) nevertheless returns an owned `ProtocolHierarchyIndex` for every query. For a document already present in the index, it executes:

```rust
return Ok(self.logical.clone());
```

That is a deep clone of the complete hierarchy, including cloned document fields and bodies, merely to read one role or relationship.

The Logs list asks `validate_task_hierarchy` once per visible row to render `[EPIC]`, `[TASK]`, or `[SUBTASK]`. At 150×46 with approximately 39 visible rows and 257 Logs totaling about 1.3 MB, one selection redraw clones the full hierarchy approximately 39 times—roughly 50 MB of transient copying.

The adapter combined two different cases under one owned return type:

1. Queries for canonical documents already in the index need only borrow `self.logical`.
2. Queries for a hypothetical replacement document need a temporary rebuilt index and must own it.

## Measured baseline and proof

A release-mode `TestBackend` profile at 150×46 against the real workspace measured:

| Component | Current |
| --- | ---: |
| Header | ~0.06 ms |
| Detail pane | ~3 ms |
| Footer | ~0.03 ms |
| Logs list | 53–62 ms |
| Complete redraw | 56–66 ms |

A temporary A/B change making `index_for` return a borrowed existing index for canonical documents and an owned rebuilt index for hypothetical replacements reduced:

| Component | Borrowing prototype |
| --- | ---: |
| Logs list | 0.45–1.6 ms |
| Complete redraw | 1.1–2.5 ms |

All temporary profiling changes were restored. These measurements are investigation evidence, not committed benchmark infrastructure.

## History

- task-141 added canonical hierarchy role context to Logs rows.
- Architecture refactor commit `342ad95` introduced the cloning adapter.
- task-226 stopped idle redraws and projected only visible rows, reducing the number of clones but retaining approximately 55 ms interaction time. Its 250 ms threshold accepted that residual cost.
- task-244 caches committed search-filter indexes. It is valid but unrelated to empty-filter navigation.

## Scope

1. Make canonical indexed queries borrow the existing `ProtocolHierarchyIndex` without cloning or rebuilding it.
2. Preserve the owned temporary-index path for hypothetical replacement documents.
3. Keep `task_role`, `relationship`, and `validate_task_hierarchy` behavior and diagnostics unchanged.
4. Add structural tests proving canonical queries use the borrowed path and hypothetical replacement queries use the owned/rebuilt path.
5. Verify the fix at the real Logs scale with an optimized 150×46 render and held `j`/`k` navigation.

`std::borrow::Cow<'_, ProtocolHierarchyIndex>` is one small suitable design because it represents borrowed-or-owned directly. A similarly narrow split between canonical and prospective query paths is acceptable if clearer. Do not add a generalized cache or new abstraction layer.

## Out of scope

- Removing hierarchy role badges or parent context from Logs.
- Input batching, key-repeat coalescing, or redraw throttling.
- Changing synchronized terminal output.
- TUI filter caching or search changes.
- Persisted caches, background work, dependencies, protocol behavior changes, or UI redesign.
- Precomputing every hierarchy role unless post-fix measurement demonstrates a remaining problem.

## Acceptance

- Canonical `ProjectHierarchy` role, relationship, and validation queries do not clone or rebuild the complete logical hierarchy.
- Prospective replacement validation preserves current semantics and uses an isolated temporary hierarchy.
- Existing hierarchy, task-update, CLI, web, and TUI behavior remains unchanged.
- Optimized 150×46 Logs redraw on the real workspace is in the measured borrowing range or otherwise demonstrates that full-hierarchy cloning is absent; record before/after evidence.
- Holding `j` or `k` in the release TUI no longer produces a long post-release backlog under ordinary terminal use.
- Focused structural tests, full Rust tests, `cargo fmt --check`, and `cargo clippy --all-targets --all-features -- -D warnings` pass.

