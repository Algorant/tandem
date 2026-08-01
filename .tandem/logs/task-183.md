---
id: task-183
type: task
title: "Fix retired ready accord action leaking through Pi tools and CLI help"
priority: "medium"
relatedFiles: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "tandem/src/main.rs"]
tags: ["pi-tandem", "accord", "bugfix", "ui"]
createdAt: "2026-07-24T12:32:57Z"
updatedAt: "2026-07-24T13:03:13Z"
accord:
  status: "accepted"
  assignee: "worker-task-183-c3d5b267"
  claimedAt: "2026-07-24T12:39:50Z"
  deliveredAt: "2026-07-24T12:59:22Z"
  deliverables: ["Commit 69a2283 integrated on main", "Core CLI usage and runtime action regression coverage", "Pi-Tandem action/schema runtime rejection coverage", "Updated Tandem and Pi-Tandem documentation"]
  validation:
    commands: ["cargo fmt --check --manifest-path tandem/Cargo.toml", "cargo test --manifest-path tandem/Cargo.toml (163 passed)", "Direct bare accord and retired ready CLI probes (both exit 2 with correct action list)", "Pi-Tandem repo read, smoke, relationship smoke, and project-local Pi runtime smoke"]
  summary: "Accepted the Tandem protocol/CLI and repository-owned Pi-Tandem fix. The shared Pi UI renderer is explicitly outside Tandem scope and will be addressed separately after this protocol release."
  evidence: ["git diff HEAD^ --check passed", "No retired accord-ready action patterns remain in repository-owned Pi-Tandem/current CLI surfaces", "Shared Pi UI custom-tools/tandem.ts remains an external follow-up decision"]
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/tests/smoke.ts", "tandem/src/main.rs", "tandem/README.md", "tandem/RELEASE.md"]
  reviewer: "user"
  updatedAt: "2026-07-24T13:03:07Z"
assignee: "worker-task-183-c3d5b267"
completedAt: "2026-07-24T13:03:13Z"
completion:
  summary: "Removed the retired ready action from Tandem CLI help and repository-owned Pi-Tandem action/schema/docs, added regression coverage, and preserved legacy persisted ready status reads. External Pi UI presentation is intentionally outside Tandem scope."
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/tests/smoke.ts", "tandem/src/main.rs", "tandem/README.md", "tandem/RELEASE.md"]
  validation: "cargo fmt --check; cargo test (163 passed); direct bare accord and retired-ready CLI probes; Pi-Tandem repo read, smoke, relationship, and runtime smoke checks"
  reviewer: "user"
---

## Description

## Bug

Tandem v0.6.3 correctly rejects `tandem accord ready`, but the retired action remains exposed by Pi-facing tooling and bare CLI help.

## Scope

- Remove `ready` from pi-tandem's public action type, tool schema, renderer metadata, prompt guidance, and tests.
- Update Pi UI Tandem tool presentation so it no longer maps or styles `ready` as a current action/status.
- Correct bare `tandem accord` usage output to list only supported actions.
- Keep legacy persisted `accord.status: ready` readable where intentionally supported; do not reintroduce it as a new action.
- Add focused regression coverage for the accepted action list and CLI help.

Do not use a local/global configuration workaround in place of the product/integration fix.
