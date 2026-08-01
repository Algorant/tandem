---
id: task-185
type: task
title: "Adopt cargo-dist changelog releases with release preflight and publication verification"
priority: "high"
relatedFiles: ["RELEASES.md", "tandem/GITHUB_RELEASE_NOTES.md", "justfile", "tandem/Cargo.toml", "tandem/dist-workspace.toml", "tandem/RELEASE.md"]
tags: ["docs", "release", "validation", "automation"]
createdAt: "2026-07-24T13:55:53Z"
updatedAt: "2026-07-26T14:25:47Z"
accord:
  status: "accepted"
  assignee: "worker-task-185-b1764b0c"
  claimedAt: "2026-07-24T14:33:15Z"
  deliveredAt: "2026-07-24T15:18:31Z"
  deliverables: ["RELEASES.md canonical release history", "just release preflight and publication verification", "release_checks fixtures for Release and AUR run selection"]
  validation:
    commands: ["cargo test --manifest-path tandem/Cargo.toml (163 passed)", "cargo fmt --check --manifest-path tandem/Cargo.toml", "cargo dist manifest notes assertion", "python3 -m unittest scripts.tests.test_release_checks -v", "Live v0.6.4 Release→AUR workflow metadata correlation"]
  summary: "Validated against the completed v0.6.5 release: preflight, cargo-dist notes, published GitHub Release/assets, AUR workflow, and installer smoke test all succeeded."
  filesChanged: ["RELEASES.md", "justfile", "scripts/release_checks.py", "scripts/tests/test_release_checks.py", "scripts/tests/fixtures/workflow-runs.json", "tandem/README.md", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md"]
  reviewer: "pi"
  updatedAt: "2026-07-26T14:25:42Z"
assignee: "worker-task-185-b1764b0c"
completedAt: "2026-07-26T14:25:47Z"
completion:
  summary: "Replaced the unused release-notes path with cargo-dist-backed RELEASES.md preflight and publication verification, proven by the v0.6.5 release."
  filesChanged: ["RELEASES.md", "scripts/release_checks.py", "scripts/tests/test_release_checks.py", "justfile", "tandem/README.md", "tandem/RELEASE.md"]
  validation: "`just release 0.6.5` passed its full local checks and preflight; GitHub Release tandem-v0.6.5 was published with curated notes and 13 assets; AUR workflow 30114305074 succeeded; clean-home installer smoke reported tandem 0.6.5."
  reviewer: "pi"
---

## Description

## Goal

Replace the unused `tandem/GITHUB_RELEASE_NOTES.md` release-note path with cargo-dist's native changelog support and make a release verifiably complete only after its published outputs are confirmed.

## Scope

- Create root `RELEASES.md` as the canonical curated release history, using parseable version headings such as `## 0.6.5` with meaningful summary/fix content.
- Configure cargo-dist to include the matching version section in the GitHub Release body alongside generated install/download information.
- Make `just release <version>` fail before tagging unless `RELEASES.md` exists, contains exactly one meaningful matching version section, and agrees with the requested version and `Cargo.toml`.
- During release validation, inspect cargo-dist's generated release announcement/manifest and assert that the GitHub body includes the curated version notes.
- After publishing, verify the GitHub Release is published (not draft/prerelease), includes the expected notes and assets, and the AUR workflow succeeds before reporting success.
- Remove or migrate the obsolete `tandem/GITHUB_RELEASE_NOTES.md` flow and update release documentation.

## Maintenance policy

Update `RELEASES.md` while preparing a release; normal task/commit/log history remains the detailed ongoing record. Drafting ahead is allowed for larger releases, but no mandatory rolling Unreleased section.
