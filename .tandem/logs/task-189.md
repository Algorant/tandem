---
id: task-189
type: task
title: "Migrate legacy priority aliases during protocol 0.2 upgrade"
priority: "critical"
relatedFiles: ["tandem/src/", "tandem/tests/cli_behavior.rs", "protocol/plan/spec.md", "RELEASES.md"]
tags: ["protocol", "upgrade", "compatibility", "migration", "hotfix"]
createdAt: "2026-07-29T02:00:10Z"
updatedAt: "2026-07-29T02:05:20Z"
accord:
  status: "accepted"
  assignee: "worker-task-189-0c6d1a70"
  claimedAt: "2026-07-29T02:00:31Z"
  deliveredAt: "2026-07-29T02:04:56Z"
  deliverables: ["Commit 921d7eb", "Upgrade migration implementation", "Normative protocol update", "v0.7.1 release notes", "Real-command regression coverage"]
  validation:
    commands: ["Independent cargo fmt --check passed", "Independent strict Clippy passed", "Independent cargo test passed: 206 unit + 6 command tests", "Focused upgrade regression covers active custom document, completed log, canonical log, comments, unknown fields, bodies, config, and events", "Pi extension syntax and smoke checks passed", "git diff --check passed"]
  constraints: ["No broad validation relaxation; logs remain immutable outside explicit upgrade", "No release or push performed"]
  summary: "Accepted after source review, independent full-suite and strict-Clippy validation, focused post-integration upgrade regression, and clean integration to main."
  evidence: ["Document snapshots and config snapshot are conflict-checked before writes", "Document priority patches occur before protocolVersion so interrupted runs remain retryable as 0.1", "Worker checkout clean"]
  filesChanged: ["tandem/src/app/project.rs", "tandem/src/cli/commands.rs", "tandem/tests/cli_behavior.rs", "protocol/plan/spec.md", "RELEASES.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-07-29T02:05:15Z"
assignee: "worker-task-189-0c6d1a70"
completedAt: "2026-07-29T02:05:20Z"
completion:
  summary: "Fixed protocol 0.2 upgrade to canonicalize legacy med/normal priorities in active documents and logs; integrated commit 921d7eb for v0.7.1."
  filesChanged: ["tandem/src/app/project.rs", "tandem/src/cli/commands.rs", "tandem/tests/cli_behavior.rs", "protocol/plan/spec.md", "RELEASES.md"]
  validation: "206 unit and 6 command tests, focused post-integration upgrade regression, formatting, strict Clippy, extension checks/smokes, and diff checks passed."
  reviewer: "orchestrator"
---

## Description

Fix Tandem v0.7.0 protocol upgrade so recognized legacy priority aliases such as `normal` and `med` are canonicalized to `medium` across active documents and completed logs. Preserve unrelated frontmatter/body content, keep upgrade explicit, add regression coverage for archived logs and mixed valid/legacy priorities, document the migration behavior, and prepare the fix for a v0.7.1 hotfix. Do not push or release within this Task.
