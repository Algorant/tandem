---
id: task-144
type: task
title: "Release canonical hierarchy hardening as Tandem v0.6.0"
priority: "high"
blockers: ["task-143"]
references: ["task-134", "decision-7", "decision-5", "task-131"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md", "justfile", ".github/workflows/release.yml", ".github/workflows/aur-tandem-bin.yml"]
tags: ["docs", "release", "validation", "hierarchy"]
createdAt: "2026-07-22T02:18:35Z"
updatedAt: "2026-07-22T04:31:13Z"
accord:
  status: "accepted"
  assignee: "shep-task-144-release-prep"
  claimedAt: "2026-07-22T04:16:57Z"
  deliveredAt: "2026-07-22T04:25:27Z"
  deliverables: ["Tandem package and lockfile bumped to 0.6.0.", "Release checklist updated consistently to tandem-v0.6.0.", "Concise v0.6.0 GitHub Release notes with Highlights, Bug fixes, and Compatibility note sections.", "Focused release commit e39fd86534b5e69c15506a6baa2b49eb2dd1b532."]
  validation:
    commands: ["Parent inspected the complete four-file diff and clean worktree.", "cargo fmt --check and all 154 Rust tests passed.", "cargo build --release, `--version`, and `version` report tandem 0.6.0.", "Frozen Bun install, docs build, 602-link check, and high-severity audit passed.", "Bun syntax check and all three pi-tandem smoke suites passed against the 0.6.0 release binary.", "Release metadata/note assertions and git diff checks passed.", "`/usr/bin/tandem` remains untouched at version 0.5.0."]
  summary: "Accepted after parent-owned publication and verification. Main and annotated tandem-v0.6.0 tag point to e39fd86; cargo-dist Release run 29891077796 succeeded; the published GitHub Release body was corrected to the curated no-installation notes; all required archives, per-artifact checksums, installer, and sha256.sum were verified; the x86_64 archive and isolated installer both report 0.6.0; AUR run 29891174478 succeeded and tandem-bin 0.6.0 checksum matches the release archive. `/usr/bin/tandem` remains untouched at 0.5.0."
  evidence: ["Commit e39fd86534b5e69c15506a6baa2b49eb2dd1b532 based directly on main 0309e2f.", "No local or remote tandem-v0.6.0 tag or GitHub Release existed during preparation.", "No install, push, tag creation, release mutation, or workflow dispatch was performed by the worker."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-22T04:30:55Z"
completedAt: "2026-07-22T04:31:13Z"
completion:
  summary: "Released and verified Tandem v0.6.0 from e39fd86: pushed main and annotated tag, published curated GitHub Release notes and cargo-dist artifacts, validated checksums/archive/isolated installer, and confirmed tandem-bin AUR 0.6.0 publication."
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md"]
  validation: "154 Rust tests and full release matrix passed. GitHub Release run 29891077796 succeeded with installer, four platform archives, four per-archive checksums, and sha256.sum; curated body matches GITHUB_RELEASE_NOTES.md. Isolated branded-installer smoke and x86_64 archive report 0.6.0. AUR run 29891174478 succeeded; AUR commit 976918d has pkgver 0.6.0 and matching checksum. /usr/bin/tandem remains unchanged at 0.5.0."
  reviewer: "parent-orchestrator"
---

## Description

Cut and verify the first release that implements decision-7 after task-143 completes.

Acceptance criteria:
- Bump the Tandem package and release documentation to 0.6.0.
- Curate concise public release notes with a dedicated Bug fixes section and no installation guidance.
- Run the complete Rust, Bun, pi-tandem, docs, link, and release validation matrix from tandem/RELEASE.md.
- Commit the release on main, create and push annotated tag tandem-v0.6.0, and push main.
- Verify the cargo-dist GitHub Release exists with installer, supported platform archives, per-artifact checksums, and sha256.sum.
- Verify the tandem-bin AUR workflow succeeds or document an actionable release blocker.
- Do not install or modify the machine-local /usr/bin/tandem binary as part of this task.
