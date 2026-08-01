---
id: task-112
type: task
title: "Document Tandem release and install workflow"
priority: "medium"
parentId: "task-108"
relatedFiles: ["tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md", "README.md", "site/"]
tags: ["docs", "release"]
createdAt: "2026-07-05T17:08:25Z"
updatedAt: "2026-07-08T00:35:55Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-08T00:07:25Z"
  deliveredAt: "2026-07-08T00:35:38Z"
  deliverables: ["Commit 3ade8a2 on branch shep/task-112-document-tandem-release-and-install-work documents cargo-dist/GitHub Actions as official binary release path.", "Docs list initial targets: Linux x86_64, Linux ARM64, macOS Intel, macOS Apple Silicon; no Windows initially.", "Docs describe GitHub Release artifacts/checksums, primary trytandem.dev/install.sh user-local installer, and x86_64-only tandem-bin AUR automation.", "Release checklist includes artifact verification, installer smoke test, and AUR automation verification."]
  validation:
    commands: ["Worker: git diff --check passed.", "Worker: stale install/release wording rg check had no matches.", "Parent: cd site && bun install --frozen-lockfile && bun run build && bun run check:links passed; build produced known non-blocking Starlight 404 warning.", "Parent: git status --short clean; ignored generated site deps/build outputs only."]
  summary: "Accepted task-112: release/install documentation meets the requested requirements and parent validation passed."
  evidence: ["Branch/worktree: shep/task-112-document-tandem-release-and-install-work @ /home/ivan/.pi/agent/worktrees/tandem/task-112-document-tandem-release-and-install-work", "Commit: 3ade8a2 Document release and install workflow"]
  filesChanged: ["README.md", "docs/quick-start/index.md", "docs/index.md", "docs/guides/docs-site.md", "docs/packaging/aur-tandem-bin.md", "site/README.md", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md"]
  reviewer: "parent"
  updatedAt: "2026-07-08T00:35:47Z"
completedAt: "2026-07-08T00:35:55Z"
completion:
  summary: "Documented Tandem release/install workflow: cargo-dist/GitHub Actions release artifacts, supported targets, checksums, trytandem.dev installer, x86_64 tandem-bin AUR automation, and release checklist verification steps."
  filesChanged: ["README.md", "docs/quick-start/index.md", "docs/index.md", "docs/guides/docs-site.md", "docs/packaging/aur-tandem-bin.md", "site/README.md", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md"]
  validation: "Accepted after parent review of commit 3ade8a2. Validation passed: cd site && bun install --frozen-lockfile && bun run build && bun run check:links."
  reviewer: "parent"
---

## Description

Update release/distribution documentation to describe the new official release process.

Requirements:
- Document cargo-dist/GitHub Actions as the official binary artifact path.
- List supported initial targets: Linux x86_64, Linux ARM64, macOS Intel, macOS Apple Silicon; no Windows initially.
- Document GitHub Release artifact/checksum expectations.
- Document primary install command via trytandem.dev/install.sh and user-local install path behavior.
- Document AUR tandem-bin automation and x86_64-only initial support.
- Update release checklist so future releases include artifact verification, install smoke test, and AUR automation verification.
