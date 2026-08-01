---
id: task-182
type: task
title: "Remove redundant ready accord status and streamline delegation claims"
priority: "high"
relatedFiles: ["protocol/plan/spec.md", "tandem/src/main.rs", "tandem/src/tui.rs", "extensions/pi-tandem/index.ts"]
tags: ["protocol", "accord", "delegation", "cli", "tui", "pi-tandem"]
createdAt: "2026-07-24T03:30:22Z"
updatedAt: "2026-07-24T03:59:04Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-07-24T03:39:04Z"
  deliveredAt: "2026-07-24T03:45:15Z"
  deliverables: ["Core accord CLI/status surface now uses claim|deliver|accept|rework|block|fail", "Legacy accord.status=ready remains readable but cannot be newly authored through the CLI", "pi-tandem action schema, docs, and smoke flow use direct claim", "Protocol/TUI guidance describes missing → claimed as the normal path"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test (162 passed)", "TANDEM_BIN=$PWD/tandem/target/debug/tandem bun extensions/pi-tandem/tests/smoke.ts", "Disposable CLI smoke: missing → claimed/in-progress succeeds; accord ready is rejected", "git diff --check"]
  summary: "User accepted removal of ready and direct claim as the ordinary accord/delegation path."
  evidence: ["Direct claim output: From missing To claimed; State todo → in-progress. Ready command output: unknown accord subcommand ready."]
  reviewer: "user"
  updatedAt: "2026-07-24T03:58:53Z"
completedAt: "2026-07-24T03:59:04Z"
completion:
  summary: "Removed ready from new accord actions/statuses; direct claim is now the normal delegation and recovery transition while legacy ready remains readable."
  filesChanged: ["tandem/src/main.rs", "tandem/src/tui.rs", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "protocol/plan/spec.md", "tandem/plan/spec.md"]
  validation: "cargo fmt --check; cargo test (162 passed); pi-tandem smoke; direct missing → claimed and ready-rejection CLI probes"
  reviewer: "user"
---

## Description

Remove `ready` as a new Tandem accord status and CLI action. Make direct `missing → claimed` the ordinary ownership transition, preserve historical ready events/documents through explicit legacy-compatible reads or migration guidance, and update protocol, CLI, TUI, tests, documentation, and pi-tandem delegation guidance. Follow with worker-launch integration that claims using the real worker identity before prompting work; do not duplicate protocol behavior in the extension.
