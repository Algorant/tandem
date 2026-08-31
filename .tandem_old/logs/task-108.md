---
id: task-108
type: task
kind: "epic"
title: "Release automation and binary install distribution"
priority: "high"
relatedFiles: ["tandem/Cargo.toml", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md", ".github/workflows/docs.yml", "site/"]
tags: ["config", "release", "distribution"]
createdAt: "2026-07-05T17:08:07Z"
updatedAt: "2026-07-08T01:09:01Z"
subtasks:
  - id: task-108-1
    title: "Configure cargo-dist release artifact workflow"
    completed: false
  - id: task-108-2
    title: "Expose branded trytandem.dev installer redirect"
    completed: false
  - id: task-108-3
    title: "Automate tandem-bin AUR updates after release"
    completed: false
  - id: task-108-4
    title: "Document release and install workflow"
    completed: false
completedAt: "2026-07-08T01:09:01Z"
completion:
  summary: "Completed release automation and binary install distribution epic. The release workflow now uses cargo-dist/GitHub Actions for official binary artifacts, the branded install URL redirects to the cargo-dist installer, tandem-bin AUR automation is implemented and validated, and release/install documentation is updated."
  filesChanged: ["tandem/Cargo.toml", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md", ".github/workflows/release.yml", ".github/workflows/aur-tandem-bin.yml", "README.md", "docs/quick-start/index.md", "docs/index.md", "docs/guides/docs-site.md", "docs/packaging/aur-tandem-bin.md", "site/README.md"]
  validation: "Child work completed and validated: task-109 configured cargo-dist release artifacts; task-114 fixed cargo-dist build profile; task-111 implemented and validated tandem-bin AUR automation; task-113 replaced branded install shim with real redirect; task-110 was closed as fulfilled by the redirect implementation; task-112 documented the release/install workflow. Live validation included GitHub Release assets, trytandem.dev/install.sh redirect, AUR workflow success, and docs build/link checks."
  reviewer: "parent"
---

## Description

Build a cohesive Tandem release/distribution workflow around cargo-dist, GitHub Release binary artifacts, a branded curl installer, and automated AUR binary package updates.

Resolved direction:
- Official release binaries are built by GitHub Actions via cargo-dist, not manually/local-only.
- Initial binary targets: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-apple-darwin, aarch64-apple-darwin.
- No Windows target initially.
- Release artifacts should include archives and checksums attached to GitHub Releases.
- Primary install UX should be a modern machine-detecting shell command from trytandem.dev, e.g. curl -fsSL https://trytandem.dev/install.sh | sh.
- Installer default path should be user-local, avoiding sudo by default, likely ~/.local/bin or cargo-dist equivalent.
- trytandem.dev/install.sh should redirect to the cargo-dist generated installer rather than maintaining a separate custom installer initially.
- AUR automation should publish/update a stable binary package named tandem-bin.
- AUR automation should run only after GitHub Release artifacts/checksums exist.
- Initial AUR architecture support should be x86_64 only.
- GitHub secret SSH key for AUR push is acceptable; user will provide it.

Out of initial scope:
- Windows artifacts.
- Manual local artifact upload as the official release path.
- Source-building AUR package automation unless later requested.
