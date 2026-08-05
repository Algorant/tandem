---
id: task-221
type: task
title: "Release Tandem 0.9.0"
priority: "high"
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "justfile"]
tags: ["chore", "release"]
createdAt: "2026-08-05T19:52:38Z"
updatedAt: "2026-08-05T20:00:35Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-08-05T19:52:42Z"
  deliveredAt: "2026-08-05T20:00:25Z"
  deliverables: ["https://github.com/Algorant/tandem/releases/tag/tandem-v0.9.0", "Annotated tag tandem-v0.9.0 pushed to origin", "Release commit e0ca00f4936f94fe5885a87a319b62e6e7f7dcd9"]
  validation:
    commands: ["just release 0.9.0 passed all local formatting, tests, release/dist builds, strict Clippy, docs build/audit, JavaScript/TypeScript checks, Pi adapter smoke tests, manifest checks, and GitHub Release verification before the non-blocking AUR step", "GitHub Release workflow 31041659899 completed successfully", "Published release is neither draft nor prerelease and contains 13 assets", "Local release binary reports tandem 0.9.0", "AUR workflow 31041967383 reached aur.archlinux.org but failed with the explicit maintenance message: The AUR is down due to maintenance. We will be back soon."]
  summary: "Accepted. Tandem v0.9.0 is published and verified. The AUR maintenance outage does not block the release."
  evidence: ["main and tandem-v0.9.0 resolve to e0ca00f4936f94fe5885a87a319b62e6e7f7dcd9", "GitHub Release URL: https://github.com/Algorant/tandem/releases/tag/tandem-v0.9.0", "Release workflow: https://github.com/Algorant/tandem/actions/runs/31041659899", "AUR maintenance run: https://github.com/Algorant/tandem/actions/runs/31041967383"]
  filesChanged: ["README.md", "RELEASES.md", "tandem/Cargo.toml", "tandem/Cargo.lock"]
  reviewer: "pi"
  updatedAt: "2026-08-05T20:00:31Z"
assignee: "pi"
completedAt: "2026-08-05T20:00:35Z"
completion:
  summary: "Released Tandem v0.9.0 with the new local read-only web interface. Pushed main and the annotated tandem-v0.9.0 tag, published and verified the GitHub Release with 13 assets, and passed the full local and cargo-dist validation. The tandem-bin AUR update is temporarily pending because AUR explicitly reports maintenance; this is recorded as a non-blocking packaging-channel outage."
  filesChanged: ["README.md", "RELEASES.md", "tandem/Cargo.toml", "tandem/Cargo.lock"]
  validation: "Full `just release 0.9.0` validation passed through GitHub Release verification; Release workflow 31041659899 succeeded; release assets and notes verified; local binary reports 0.9.0. AUR workflow 31041967383 failed only because aur.archlinux.org reports maintenance."
---

## Description

Prepare and publish Tandem v0.9.0 for the read-only local web MVP. Update package metadata and curated release notes, run the full release validation, push the annotated tag, verify the GitHub Release, and record any temporarily unavailable packaging channel as non-blocking.
