---
id: task-223
type: task
title: "Prepare Tandem v0.10.0 Papercuts release"
state: "in-progress"
priority: "high"
references: ["task-222", "decision-5"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "README.md", "tandem/RELEASE.md", "justfile", "scripts/release_checks.py"]
tags: ["chore", "release", "papercuts"]
createdAt: "2026-08-10T12:46:07Z"
updatedAt: "2026-08-10T12:46:12Z"
accord:
  status: "claimed"
  assignee: "pi"
  claimedAt: "2026-08-10T12:46:12Z"
  updatedAt: "2026-08-10T12:46:12Z"
assignee: "pi"
---

## Description

## Outcome

Prepare a clean, fully validated Tandem v0.10.0 release candidate for the Papercuts inbox MVP.

## Scope

- Update `tandem/Cargo.toml` and `tandem/Cargo.lock` to `0.10.0`.
- Add concise curated `0.10.0` public notes to `RELEASES.md` following decision-5.
- Update current source-install examples and release capability documentation.
- Build and manually dogfood the release binary.
- Run the complete Rust, cargo-dist, pi-tandem, documentation, JavaScript, Bun audit, and diff validation suite.
- Verify the repository is clean and ready for `just release 0.10.0`.

## Publication boundary

This Task begins release preparation and validation. Do not create or push the tag, push `main`, publish the GitHub Release, update AUR, or install the published release until the product owner reviews the release candidate and explicitly approves publication.

## Proposed public positioning

Tandem v0.10.0 introduces Papercuts: a lightweight project inbox for preserving small, non-blocking friction without interrupting active work. It includes CLI commands, global search, loose Task and Decision references, the thin `pi-tandem` tool, and complete documentation. Existing workspaces require no migration.

## Acceptance criteria

1. Package and lockfile versions are `0.10.0`.
2. `RELEASES.md` contains exactly one meaningful `## 0.10.0` section with no installation guidance.
3. Release notes use Features, Improvements, and Compatibility sections and describe only shipped behavior.
4. Release documentation lists the Papercut CLI and optional `.tandem/papercuts/` storage.
5. The release build completes add/list/show/search/resolve Papercut dogfood in a temporary workspace.
6. All commands required by `tandem/RELEASE.md` and `just release` pass without tagging or publishing.
7. cargo-dist includes the curated notes and expected release plan.
8. The final checkout is clean and the exact publication command is presented for approval.
