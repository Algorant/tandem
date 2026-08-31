---
id: task-145
type: task
title: "Research idiomatic modularization of large Rust CLI and TUI modules"
priority: "medium"
blockers: ["task-144"]
references: ["task-134", "decision-7"]
relatedFiles: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/"]
tags: ["tui", "architecture", "refactor", "research", "rust"]
createdAt: "2026-07-22T03:35:11Z"
updatedAt: "2026-07-22T18:03:41Z"
accord:
  status: "accepted"
  assignee: "shep-task-145-research"
  claimedAt: "2026-07-22T04:33:21Z"
  deliveredAt: "2026-07-22T18:03:18Z"
  deliverables: ["`tandem/plan/modularization-research.md`: audited current architecture, alternatives, preferred staged approach, target module map, invariants, regression coverage, risks, non-goals, and measurable completion criteria.", "Focused amended commit `8a17d91e8b5cbcc584a53d138f7e03d322cffb04` based directly on current main."]
  validation:
    commands: ["`cargo fmt --manifest-path tandem/Cargo.toml -- --check` passed.", "`cargo test --manifest-path tandem/Cargo.toml --quiet` passed: 154 passed, 0 failed, 0 ignored.", "`git diff --check main...8a17d91e8b5cbcc584a53d138f7e03d322cffb04` passed.", "Independent scripts reproduced all documented source line counts, test counts, TuiApp method/field counts, churn counts, and git log -L hotspot counts.", "Commit changes only `tandem/plan/modularization-research.md`; worker worktree is clean.", "Strict Clippy remains blocked only by documented pre-existing diagnostics; this research commit changes no Rust code."]
  summary: "Accepted task-145 after independent parent review, correction of the sole stale reference, clean fast-forward integration as 8a17d91, exact scope verification, reproduced quantitative evidence, passing formatting and 154 Rust tests, and explicit user validation."
  evidence: ["Reviewed worker handoff and full 542-line deliverable.", "Fetched and verified the cited Rust and Ratatui architecture references.", "User explicitly requested task-145 be marked validated and complete after reviewing the synthesis."]
  filesChanged: ["tandem/plan/modularization-research.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-22T18:03:31Z"
completedAt: "2026-07-22T18:03:41Z"
completion:
  summary: "Completed and integrated the research-only idiomatic Rust modularization recommendation as 8a17d91; the audited document now provides the verified basis for the separate refactor specification, architecture decision, and future Epic decomposition."
  filesChanged: ["tandem/plan/modularization-research.md"]
  validation: "Parent independently verified the one-file documentation scope, reproduced line/test/churn/hotspot measurements, confirmed external references, passed cargo fmt, 154 Rust tests, and git diff checks; user explicitly validated and requested completion."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Investigate the oversized `tandem/src/main.rs` and `tandem/src/tui.rs` modules and recommend practical, idiomatic Rust refactoring strategies without changing production behavior in this task.

## Research scope

- Inventory the responsibilities, major symbol clusters, dependency directions, shared state, test placement, and coupling hotspots in both files and the existing `tandem/src/tui/` module tree.
- Establish a quantitative baseline (line counts, major implementation/test regions, and especially costly or high-churn seams).
- Evaluate idiomatic Rust options such as a thin binary entry point plus library/application modules, cohesive command/domain/persistence boundaries, TUI state/input/update/render decomposition, narrow `pub(crate)` APIs, and colocated versus integration tests.
- Prefer concrete modules and data-flow boundaries over speculative traits or framework-like abstraction; identify where traits, generics, or new types are actually justified.
- Preserve Tandem conventions: one `tandem/` binary crate in v0, no root workspace or crate proliferation, protocol logic separate from TUI rendering and Pi adapters, canonical shared hierarchy APIs rather than duplicated inference, and small reviewable changes.
- Compare at least two plausible plans of attack, including trade-offs, migration risk, merge-conflict risk, compile/test implications, and how each avoids a large rewrite.

## Deliverable and acceptance criteria

- Produce a written research recommendation with a proposed target module map and ownership boundaries.
- Recommend one preferred staged approach, including an initial low-risk extraction and an ordered sequence of independently reviewable follow-up tasks.
- Identify invariants and regression coverage required before and after each stage, including CLI output and TUI behavior.
- Call out code that should deliberately remain together and abstractions that should not be introduced yet.
- Suggest measurable completion criteria for the eventual refactor.
- Make no production-code refactor as part of this research task.
