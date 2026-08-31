---
id: task-109
type: task
title: "Configure cargo-dist GitHub Release binary artifacts"
priority: "high"
parentId: "task-108"
relatedFiles: ["tandem/Cargo.toml", ".github/workflows/", "tandem/RELEASE.md"]
tags: ["config", "release", "cargo-dist"]
createdAt: "2026-07-05T17:08:25Z"
updatedAt: "2026-07-05T21:59:59Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-07-05T21:24:37Z"
  deliveredAt: "2026-07-05T21:31:08Z"
  deliverables: ["Added root dist-workspace.toml with cargo:tandem member, cargo-dist 0.32.0, GitHub CI, shell installer, and four targets: Linux x86_64/aarch64 plus macOS x86_64/aarch64.", "Generated .github/workflows/release.yml with cargo-dist release jobs that plan, build local/global artifacts, upload archives/checksums, and create the GitHub Release on tag push.", "Updated just release flow to stop manually creating a GitHub Release after pushing the tag, avoiding conflict with cargo-dist.", "Updated tandem/RELEASE.md to describe cargo-dist-created artifacts and tag-triggered release automation."]
  validation:
    commands: ["cargo-dist generate --check: passed using cargo-dist 0.32.0 installed in /tmp/cargo-dist-home/bin.", "cargo-dist plan --tag tandem-v0.4.1 --no-local-paths: passed and listed tandem-installer.sh, sha256.sum, source tarball, and all four platform archives with sha256 files.", "Python YAML parse of .github/workflows/release.yml: passed.", "git diff --check: passed."]
  summary: "Human validated task-109 cargo-dist release artifact configuration."
  evidence: ["Commit 70c936d Configure cargo-dist release artifacts.", "Working tree clean after commit."]
  filesChanged: ["dist-workspace.toml", ".github/workflows/release.yml", "justfile", "tandem/RELEASE.md"]
  updatedAt: "2026-07-05T21:59:44Z"
completedAt: "2026-07-05T21:59:59Z"
completion:
  summary: "Configured cargo-dist GitHub Release binary artifact automation for Tandem and updated release flow/docs."
  validation: "User validated. Evidence: commit 70c936d; cargo-dist generate --check; cargo-dist plan --tag tandem-v0.4.1 --no-local-paths; YAML parse of .github/workflows/release.yml; git diff --check."
  reviewer: "Algorant"
---

## Description

Add cargo-dist-based release automation for Tandem. Official releases should be built by GitHub Actions and attached to GitHub Releases.

Requirements:
- Configure cargo-dist for the Rust binary crate under tandem/.
- Initial targets: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-apple-darwin, aarch64-apple-darwin.
- Do not include Windows initially.
- Produce release archives and checksums.
- Trigger from Tandem release tags/GitHub Releases consistent with existing tandem-v* release naming.
- Keep workflow compatible with the monorepo layout where Cargo.toml is in tandem/.
- Validate generated workflow/config where practical without cutting a real release.
