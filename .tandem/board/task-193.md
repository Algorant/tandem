---
id: task-193
type: task
title: "Review AUR tandem-bin version suffix handling"
state: todo
priority: "medium"
relatedFiles: ["docs/packaging/aur-tandem-bin.md", ".github/workflows/aur-tandem-bin.yml", "RELEASES.md"]
tags: ["config", "aur", "release"]
createdAt: "2026-08-01T00:42:02Z"
updatedAt: "2026-08-01T00:42:02Z"
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
