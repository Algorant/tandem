---
id: task-215
type: task
title: "Release Tandem 0.8.4"
state: "in-progress"
priority: "high"
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/RELEASE.md"]
tags: ["release"]
createdAt: "2026-08-05T18:03:36Z"
updatedAt: "2026-08-05T18:11:50Z"
accord:
  status: "blocked"
  assignee: "orchestrator"
  claimedAt: "2026-08-05T18:03:43Z"
  evidence: ["GitHub Release workflow 31033177672 succeeded for tandem-v0.8.4.", "Published release: https://github.com/Algorant/tandem/releases/tag/tandem-v0.8.4", "Branded installer smoke succeeded and `/home/ivan/.cargo/bin/tandem --version` reported `tandem 0.8.4`.", "AUR workflow 31033480566 failed while cloning ssh://aur@aur.archlinux.org/tandem-bin.git with: `The AUR is down due to maintenance. We will be back soon.`", "AUR web package remains at tandem-bin 0.8.1-1, last updated 2026-07-31."]
  note: "Retry the AUR workflow for tandem-v0.8.4 after Arch restores AUR Git access. Do not create or reuse another 0.8.4 tag; the GitHub Release and artifacts are already published."
  reason: "GitHub Release 0.8.4 and its assets published successfully, and the branded installer installed tandem 0.8.4. The required AUR workflow still fails because AUR SSH Git access returns the Arch maintenance shutdown message, so the canonical release cannot be marked successful yet."
  updatedAt: "2026-08-05T18:11:50Z"
assignee: "orchestrator"
---

## Description

Prepare, validate, publish, and smoke-test Tandem 0.8.4. Include the documentation-site overhaul, framework-neutral agent commit guidance, and built-in BUG/FEAT/CHORE Board badges. Run the canonical `just release 0.8.4` workflow, verify GitHub assets and AUR automation, and test the branded installer. If the 0.8.4 AUR publication succeeds, use that evidence to resolve the AUR-only blocker on task-195 for release 0.8.3.
