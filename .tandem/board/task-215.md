---
id: task-215
type: task
title: "Release Tandem 0.8.4"
state: "in-progress"
priority: "high"
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/RELEASE.md"]
tags: ["release"]
createdAt: "2026-08-05T18:03:36Z"
updatedAt: "2026-08-05T18:03:43Z"
accord:
  status: "claimed"
  assignee: "orchestrator"
  claimedAt: "2026-08-05T18:03:43Z"
  updatedAt: "2026-08-05T18:03:43Z"
assignee: "orchestrator"
---

## Description

Prepare, validate, publish, and smoke-test Tandem 0.8.4. Include the documentation-site overhaul, framework-neutral agent commit guidance, and built-in BUG/FEAT/CHORE Board badges. Run the canonical `just release 0.8.4` workflow, verify GitHub assets and AUR automation, and test the branded installer. If the 0.8.4 AUR publication succeeds, use that evidence to resolve the AUR-only blocker on task-195 for release 0.8.3.
