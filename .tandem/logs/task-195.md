---
id: task-195
type: task
title: "Release Tandem 0.8.3"
priority: "high"
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/RELEASE.md"]
tags: ["release"]
createdAt: "2026-08-03T21:11:10Z"
updatedAt: "2026-08-05T18:40:02Z"
accord:
  status: "accepted"
  assignee: "orchestrator"
  claimedAt: "2026-08-03T21:11:14Z"
  deliveredAt: "2026-08-05T18:39:55Z"
  deliverables: ["Published tandem-v0.8.3 annotated tag and GitHub Release.", "Release assets and curated release notes published by the successful Release workflow."]
  validation:
    commands: ["GitHub Release workflow 30853644703 succeeded.", "GitHub Release tandem-v0.8.3 is published and non-draft/non-prerelease."]
  summary: "Tandem 0.8.3 was published successfully to GitHub with its release assets. The AUR update is excluded from release completion by explicit project-owner decision because AUR Git access is temporarily unavailable."
  evidence: ["AUR workflow 30853877291 failed only because Arch temporarily disabled AUR SSH Git access.", "Project owner explicitly decided on 2026-08-05 that temporary AUR downtime is not release-blocking and authorized completion."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
  reviewer: "project-owner"
  note: "Accepted by explicit user direction. The GitHub release succeeded; temporary AUR maintenance is recorded as a non-blocking external condition."
  updatedAt: "2026-08-05T18:39:58Z"
assignee: "orchestrator"
completedAt: "2026-08-05T18:40:02Z"
completion:
  summary: "Published Tandem 0.8.3 with a successful GitHub Release and assets. The AUR update did not publish because Arch temporarily disabled AUR Git access; by project-owner decision, this external outage is recorded but is not release-blocking."
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
  validation: "GitHub Release workflow 30853644703 succeeded and tandem-v0.8.3 is published. AUR workflow 30853877291 was unavailable due to temporary Arch maintenance and is explicitly non-blocking."
  reviewer: "project-owner"
---

## Description

Prepare, validate, publish, and smoke-test Tandem 0.8.3. Update package version and curated release notes, run the canonical `just release 0.8.3` workflow, verify GitHub assets and AUR automation, and test the branded installer.
