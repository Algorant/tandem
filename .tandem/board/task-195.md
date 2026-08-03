---
id: task-195
type: task
title: "Release Tandem 0.8.3"
state: "in-progress"
priority: "high"
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/RELEASE.md"]
tags: ["release"]
createdAt: "2026-08-03T21:11:10Z"
updatedAt: "2026-08-03T21:17:17Z"
accord:
  status: "blocked"
  assignee: "orchestrator"
  claimedAt: "2026-08-03T21:11:14Z"
  evidence: ["Release workflow 30853644703 succeeded", "GitHub Release https://github.com/Algorant/tandem/releases/tag/tandem-v0.8.3 is published with 13 expected assets", "Installer smoke returned tandem 0.8.3", "AUR workflow 30853877291 failed twice during git clone with: The AUR is down due to maintenance."]
  reason: "GitHub Release tandem-v0.8.3 is published and installer smoke passes, but the required AUR workflow failed twice because aur.archlinux.org reports scheduled maintenance. Retry the failed workflow when AUR returns."
  updatedAt: "2026-08-03T21:17:17Z"
assignee: "orchestrator"
---

## Description

Prepare, validate, publish, and smoke-test Tandem 0.8.3. Update package version and curated release notes, run the canonical `just release 0.8.3` workflow, verify GitHub assets and AUR automation, and test the branded installer.
