---
id: task-215
type: task
title: "Release Tandem 0.8.4"
priority: "high"
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/RELEASE.md"]
tags: ["release"]
createdAt: "2026-08-05T18:03:36Z"
updatedAt: "2026-08-05T18:40:14Z"
accord:
  status: "accepted"
  assignee: "orchestrator"
  claimedAt: "2026-08-05T18:03:43Z"
  deliveredAt: "2026-08-05T18:40:07Z"
  deliverables: ["Published tandem-v0.8.4 annotated tag and GitHub Release with expected platform assets and checksums.", "Published curated 0.8.4 release notes.", "Verified the branded installer installs tandem 0.8.4."]
  validation:
    commands: ["Canonical local release validation passed before publication.", "GitHub Release workflow 31033177672 succeeded.", "`curl -fsSL https://trytandem.dev/install.sh | sh` succeeded and installed `tandem 0.8.4`."]
  summary: "Tandem 0.8.4 was validated, published to GitHub with all expected assets, and installed successfully through the branded installer. Temporary AUR downtime is recorded as non-blocking by project-owner decision."
  evidence: ["Published release: https://github.com/Algorant/tandem/releases/tag/tandem-v0.8.4", "AUR workflow 31033480566 failed only because Arch temporarily disabled AUR SSH Git access.", "Project owner explicitly decided on 2026-08-05 that temporary AUR downtime is not release-blocking and authorized completion."]
  filesChanged: ["README.md", "RELEASES.md", "tandem/Cargo.toml", "tandem/Cargo.lock"]
  reviewer: "project-owner"
  note: "Accepted by explicit user direction. GitHub publication, assets, release notes, and installer smoke succeeded. Temporary AUR maintenance remains logged as non-blocking."
  updatedAt: "2026-08-05T18:40:10Z"
assignee: "orchestrator"
completedAt: "2026-08-05T18:40:14Z"
completion:
  summary: "Published Tandem 0.8.4 with successful validation, GitHub Release assets, curated notes, and branded installer smoke. The AUR update did not publish because Arch temporarily disabled AUR Git access; by project-owner decision, this external outage is recorded but is not release-blocking."
  filesChanged: ["README.md", "RELEASES.md", "tandem/Cargo.toml", "tandem/Cargo.lock"]
  validation: "Local release validation passed; GitHub Release workflow 31033177672 succeeded; branded installer installed tandem 0.8.4. AUR workflow 31033480566 was unavailable due to temporary Arch maintenance and is explicitly non-blocking."
  reviewer: "project-owner"
---

## Description

Prepare, validate, publish, and smoke-test Tandem 0.8.4. Include the documentation-site overhaul, framework-neutral agent commit guidance, and built-in BUG/FEAT/CHORE Board badges. Run the canonical `just release 0.8.4` workflow, verify GitHub assets and AUR automation, and test the branded installer. If the 0.8.4 AUR publication succeeds, use that evidence to resolve the AUR-only blocker on task-195 for release 0.8.3.
