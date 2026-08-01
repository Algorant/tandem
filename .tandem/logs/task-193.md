---
id: task-193
type: task
title: "Review AUR tandem-bin version suffix handling"
priority: "medium"
relatedFiles: ["docs/packaging/aur-tandem-bin.md", ".github/workflows/aur-tandem-bin.yml", "RELEASES.md"]
tags: ["config", "aur", "release"]
createdAt: "2026-08-01T00:42:02Z"
updatedAt: "2026-08-01T02:34:38Z"
accord:
  status: "accepted"
  assignee: "worker-task-193-ef3a7a2d"
  claimedAt: "2026-08-01T02:31:56Z"
  deliveredAt: "2026-08-01T02:34:25Z"
  deliverables: ["Evidence-backed trace from `tandem-v0.8.1` through the GitHub Release, AUR workflow, generated PKGBUILD/.SRCINFO, AUR RPC output, and expected package filename.", "Comparison of AUR publications for upstream versions 0.8.0 and 0.8.1.", "Root-cause conclusion and explicit no-change recommendation."]
  validation:
    commands: ["Verified `.github/workflows/aur-tandem-bin.yml` strips only the `tandem-v` tag prefix, emits `pkgver=0.8.1`, and separately sets `pkgrel=1`.", "Verified Arch PKGBUILD(5) and ArchWiki define pkgrel as the distribution-specific release number, normally reset to 1 for each upstream release.", "Verified live AUR RPC reports `Version: 0.8.1-1`.", "Compared AUR commits for 0.8.0 and 0.8.1, regenerated `.SRCINFO`, checked the package filename, release URLs, and matching checksums."]
  summary: "Investigation confirms `0.8.1-1` is the correct Arch full package version: Tandem upstream `pkgver=0.8.1` plus distribution package release `pkgrel=1`. No duplicated suffix or release automation defect was found, so no repository change is required."
  evidence: ["https://man.archlinux.org/man/PKGBUILD.5.en", "https://wiki.archlinux.org/title/PKGBUILD", "https://aur.archlinux.org/rpc/v5/info/tandem-bin", ".github/workflows/aur-tandem-bin.yml"]
  reviewer: "orchestrator"
  note: "Accepted after independent review. The repository workflow explicitly keeps upstream `pkgver` separate from Arch `pkgrel=1`; authoritative Arch documentation and live AUR RPC confirm `0.8.1-1` is expected. No integration is needed because the clean Worker branch contains no file changes."
  updatedAt: "2026-08-01T02:34:30Z"
assignee: "worker-task-193-ef3a7a2d"
completedAt: "2026-08-01T02:34:38Z"
completion:
  summary: "Confirmed the trailing `-1` is normal Arch package-release metadata, not a Tandem versioning defect. Tandem publishes upstream version `0.8.1`; the AUR PKGBUILD sets `pkgver=0.8.1` and `pkgrel=1`, so paru and AUR correctly display the full package version as `0.8.1-1`. No code, workflow, documentation, release, or AUR update is required."
  validation: "Reviewed the Worker evidence; inspected `.github/workflows/aur-tandem-bin.yml`; confirmed tags and GitHub Release assets contain no extra suffix; checked Arch PKGBUILD(5), ArchWiki, and live AUR RPC; verified the Worker branch is clean with no changes to integrate."
  reviewer: "orchestrator"
---

## Description

## Context

`paru tandem-bin` reports AUR version `0.8.1-1` (installed: `0.8.0-1`). Review why AUR releases include the trailing `-1` and determine whether this is expected Arch package release metadata or an error in Tandem's release automation.

## Investigation

- Trace the version from the Tandem git tag/GitHub release through the AUR workflow and generated `PKGBUILD`/`.SRCINFO` metadata.
- Compare the current `0.8.1` publication with the previous `0.8.0` publication.
- Confirm the Arch `pkgver-pkgrel` convention and whether `pkgrel=1` correctly produces `0.8.1-1`.
- Check for any accidental duplicate suffix in tags, artifact names, package metadata, or automation inputs.

## Deliverable

Document the root cause and expected user-visible version. If behavior is wrong, propose the smallest fix and validation plan. Do not publish a new AUR release as part of the investigation unless separately authorized.
