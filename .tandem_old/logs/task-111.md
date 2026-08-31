---
id: task-111
type: task
title: "Automate tandem-bin AUR updates after releases"
priority: "high"
parentId: "task-108"
relatedFiles: [".github/workflows/", "tandem/RELEASE.md"]
tags: ["release", "aur", "automation"]
createdAt: "2026-07-05T17:08:25Z"
updatedAt: "2026-07-07T23:56:27Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-06T03:15:22Z"
  deliveredAt: "2026-07-06T03:44:10Z"
  deliverables: ["Added .github/workflows/aur-tandem-bin.yml triggered by successful Release workflow runs and manual dispatch with a tag.", "Workflow downloads tandem-x86_64-unknown-linux-gnu.tar.xz and sha256.sum from the GitHub Release, extracts the artifact checksum, generates PKGBUILD/.SRCINFO in an Arch container, and pushes to ssh://aur@aur.archlinux.org/tandem-bin.git with AUR_SSH_PRIVATE_KEY.", "Added docs/packaging/aur-tandem-bin.md documenting scope, required secrets, AUR remote setup, triggering behavior, and manual recovery.", "Updated tandem/RELEASE.md with AUR automation notes.", "Worker branch/worktree: shep/task-111-automate-tandem-bin-aur-updates-after-re at /home/ivan/.pi/agent/worktrees/tandem/task-111-automate-tandem-bin-aur-updates-after-re.", "Commit: 68a4f59 Automate tandem-bin AUR updates."]
  validation:
    commands: ["Python YAML parse of .github/workflows/aur-tandem-bin.yml: passed.", "Extracted embedded PKGBUILD template and ran bash -n /tmp/PKGBUILD.tandem-bin: passed.", "git diff --check: passed.", "git status --short in worker worktree: clean.", "actionlint unavailable in environment; not run."]
  summary: "Accepted task-111: tandem-bin AUR automation is implemented, pushed, and end-to-end validated against the initialized AUR repo for tandem-v0.4.2."
  evidence: ["Reviewed commit/diff from worktree branch shep/task-111-automate-tandem-bin-aur-updates-after-re.", "Commit 68a4f59.", "No secrets committed."]
  filesChanged: [".github/workflows/aur-tandem-bin.yml", "docs/packaging/aur-tandem-bin.md", "tandem/RELEASE.md"]
  updatedAt: "2026-07-07T23:56:21Z"
completedAt: "2026-07-07T23:56:27Z"
completion:
  summary: "Automated tandem-bin AUR updates after releases and validated successful AUR workflow run."
  validation: "GitHub Actions AUR workflow run 28836936214 completed successfully on workflow_dispatch for tag tandem-v0.4.2. Evidence: downloaded release artifact/checksums, configured AUR SSH key, generated PKGBUILD/.SRCINFO, committed and pushed AUR update. AUR remote refs/heads/master resolves at a784eca7f9f3d3fc76c9d54926df01c6d0a86ad1. Release assets for tandem-v0.4.2 exist and branded install redirect was validated."
  reviewer: "pi"
---

## Description

Automate maintenance of the AUR binary package for Tandem.

Requirements:
- Target package name: tandem-bin.
- Initial architecture: x86_64 only.
- Consume the published GitHub Release Linux x86_64 binary artifact and checksum; do not build from source in the AUR package.
- Run automation only after GitHub Release artifacts/checksums exist.
- Use an SSH key stored as a GitHub secret for pushing to AUR; user will provide the key.
- Generate/update PKGBUILD and .SRCINFO.
- Document required GitHub secret names, AUR remote setup, and manual recovery steps.
- Avoid committing secrets.
