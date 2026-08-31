---
id: task-237
type: task
title: "Audit and accelerate the `just release` workflow"
priority: "medium"
effort: "medium"
references: ["task-236", "task-185"]
relatedFiles: ["justfile", "scripts/release_checks.py", "scripts/tests/test_release_checks.py", "tandem/RELEASE.md", "tandem/dist-workspace.toml"]
tags: ["release", "performance", "automation"]
createdAt: "2026-08-21T21:41:14Z"
updatedAt: "2026-08-22T23:07:50Z"
accord:
  status: "accepted"
  assignee: "worker-task-237-786b832e"
  claimedAt: "2026-08-22T22:55:19Z"
  deliveredAt: "2026-08-22T23:07:30Z"
  validation:
    commands: ["cargo fmt --check", "cargo clippy --all-targets --all-features -- -D warnings", "cargo test", "cargo test --release", "cargo build --profile dist", "python release-check tests", "just --dry-run release 0.10.3", "cargo rustc --release -- --print cfg confirms debug_assertions present", "cargo rustc --profile dist -- --print cfg confirms debug_assertions absent"]
  summary: "Release gate now runs `cargo test --release` so the required release build reuses the test compilation. `[profile.release]` re-enables debug-assertions and overflow-checks; `[profile.dist]` explicitly disables both, leaving the shipped artifact unchanged. Cold cargo stages 167.6s to 114.1s with validation strength preserved. Baseline method, timing table, and deferred opportunities documented in RELEASE.md."
  filesChanged: ["justfile", "tandem/Cargo.toml", "tandem/RELEASE.md"]
  note: "Orchestrator-verified. Independently confirmed debug_assertions present under --release and absent under --profile dist via cargo rustc --print cfg, and reran cargo test --release. Acceptance criteria 1-6 met: reproducible baseline method, evidence-backed bottleneck (duplicate dependency-graph compilation), implemented speedup, before/after timings, release guarantees intact, deferred opportunities recorded in RELEASE.md."
  updatedAt: "2026-08-22T23:07:44Z"
assignee: "worker-task-237-786b832e"
completedAt: "2026-08-22T23:07:50Z"
completion:
  summary: "Release gate switched to `cargo test --release` so the required release build reuses the test compilation, with `[profile.release]` re-enabling debug-assertions/overflow-checks and `[profile.dist]` explicitly disabling both so the shipped artifact is unchanged. Cold cargo stages 167.6s to 114.1s with validation strength preserved. Merged to main as a1f7f81."
---

## Description

## Goal

Reduce the end-to-end time and avoidable friction of `just release <version>` without weakening release correctness or publication verification.

## Scope

- Measure the current release path stage by stage on representative warm and cold runs.
- Identify the main time sinks, duplicated validation, serial work that can safely run in parallel, ineffective caching, unnecessary rebuilds, network waits, and manual steps.
- Separate mandatory safety checks from redundant or misplaced work.
- Propose and implement the smallest high-confidence speedups where practical.
- Document larger follow-up opportunities with expected benefit, risk, and implementation cost.

## Acceptance criteria

1. A reproducible baseline records total duration and major stage timings.
2. The dominant bottlenecks are identified with evidence rather than guesswork.
3. Safe improvements are implemented and covered by relevant tests or validation.
4. Before/after timings quantify the result.
5. Release guarantees remain intact, including version and notes checks, build/test validation, artifact publication verification, installer verification, and downstream packaging checks.
6. Any deferred opportunities are recorded as concrete follow-up tasks or recommendations.
