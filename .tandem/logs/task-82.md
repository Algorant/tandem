---
id: task-82
type: task
title: "Add Epic protocol and CLI core support"
priority: "high"
parentId: "task-80"
relatedFiles: ["protocol/plan/spec.md", "docs/protocol/index.md", "docs/cli/index.md", "tandem/src/main.rs", "tandem/README.md"]
tags: ["protocol", "epic", "cli"]
createdAt: "2026-07-01T18:05:30Z"
updatedAt: "2026-07-01T18:30:24Z"
accord:
  status: "accepted"
  assignee: "shep-epic-core"
  claimedAt: "2026-07-01T18:07:40Z"
  deliveredAt: "2026-07-01T18:28:54Z"
  deliverables: ["Branch: hp/task80-epic-core in ../tandem-worktrees/hp-task80-epic-core", "CLI supports `--kind epic` for add/update and displays KIND in list/search/show/JSON where relevant", "Protocol/CLI specs updated for lightweight Epic semantics"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml --check: pass", "cd tandem && cargo test --quiet: 100 passed", "git diff --check: pass", "Worker temp workspace smoke passed add/list/show/search/update and invalid `--kind feature` rejection", "git diff stat: 3 files, 153 insertions, 31 deletions"]
  summary: "Accepted verified Epic protocol/CLI core support. Evidence includes passing Rust tests, rustfmt check, diff check, and worker temp-workspace CLI smoke for add/update/list/show/search plus invalid kind rejection."
  evidence: ["shep_check task-82 showed worker done and no blockers", "Local verification run in /home/ivan/dev/projects/tandem-worktrees/hp-task80-epic-core"]
  filesChanged: ["protocol/plan/spec.md", "tandem/plan/spec.md", "tandem/src/main.rs"]
  updatedAt: "2026-07-01T18:30:19Z"
completedAt: "2026-07-01T18:30:24Z"
completion:
  summary: "Completed Epic protocol and CLI core support after verification."
  validation: "Verified branch hp/task80-epic-core with cargo fmt --check, cargo test --quiet (100 passed), git diff --check, and worker temp CLI smoke for add/update/list/show/search plus invalid kind rejection."
  reviewer: "orchestrator"
---

## Description

Implement the core lightweight Epic model for task documents: specify optional `kind: epic`, support creating/updating/displaying the field in CLI flows, preserve `type: task`, and add focused validation/tests. Keep scope minimal: no separate `tandem epic` namespace, no `epic-N` IDs, and no dedicated epic board.
