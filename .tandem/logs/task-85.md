---
id: task-85
type: task
title: "Strengthen Decision ADR protocol and CLI core"
priority: "high"
parentId: "task-81"
relatedFiles: ["protocol/plan/spec.md", "docs/protocol/index.md", "docs/cli/index.md", "tandem/src/main.rs"]
tags: ["protocol", "decision", "adr", "cli"]
createdAt: "2026-07-01T18:05:47Z"
updatedAt: "2026-07-01T20:19:58Z"
accord:
  status: "accepted"
  assignee: "shep-decision-core"
  claimedAt: "2026-07-01T18:08:04Z"
  deliveredAt: "2026-07-01T18:29:19Z"
  deliverables: ["Branch: hp/task81-decision-core in ../tandem-worktrees/hp-task81-decision-core", "Decision metadata/status/date/deciders/context/consequences/alternatives/supersession support while keeping `decision` canonical", "CLI and pi-tandem smoke coverage for ADR metadata"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml --check: pass", "cargo test --manifest-path tandem/Cargo.toml --quiet: 103 passed", "bun extensions/pi-tandem/tests/smoke.ts: pass", "git diff --check: pass", "Worker temp CLI smoke passed for decision add/list/show with ADR metadata; invalid `--status todo` correctly failed", "git diff stat: 8 files, 495 insertions, 30 deletions"]
  summary: "Objective non-visual protocol/CLI core work accepted based on recorded automated validation evidence; no human visual/product validation required."
  evidence: ["shep_check task-85 showed worker done and no blockers", "Local verification run in /home/ivan/dev/projects/tandem-worktrees/hp-task81-decision-core"]
  filesChanged: ["extensions/pi-tandem/README.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/tests/smoke.ts", "protocol/plan/spec.md", "tandem/plan/spec.md", "tandem/src/main.rs", "tandem/src/tui/decisions.rs"]
  reviewer: "Algorant/orchestrator"
  updatedAt: "2026-07-01T20:19:48Z"
completedAt: "2026-07-01T20:19:58Z"
completion:
  summary: "Completed ADR-compatible Decision protocol and CLI core work. Accepted as objective non-visual work with recorded automated validation evidence."
  validation: "Recorded validation evidence: cargo fmt --check pass; cargo test --quiet 103 passed; pi-tandem smoke pass; git diff --check pass; CLI smoke for decision add/list/show metadata passed; invalid --status todo correctly failed."
  reviewer: "Algorant/orchestrator"
---

## Description

Define and implement the core ADR-compatible Decision model while keeping `decision` as the canonical Tandem term: recommended metadata/status/date/deciders/context/consequences/alternatives/supersession fields, CLI add/show/list handling where needed, and tests that preserve the distinction from task workflow state.
