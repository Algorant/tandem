---
id: task-195
type: task
title: "Release Tandem 0.8.3"
state: "in-progress"
priority: "high"
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/RELEASE.md"]
tags: ["release"]
createdAt: "2026-08-03T21:11:10Z"
updatedAt: "2026-08-03T21:29:47Z"
accord:
  status: "blocked"
  assignee: "orchestrator"
  claimedAt: "2026-08-03T21:11:14Z"
  evidence: ["Official aur-general message: https://lists.archlinux.org/archives/list/aur-general@lists.archlinux.org/message/YPJ3FQYJTJXXY3RUXCYLMHUKHLIUNVFF/ says: We have now disabled pushes altogether as well for the moment, while we handle the situation.", "AUR workflow 30853877291 failed twice at SSH git clone with: The AUR is down due to maintenance. We will be back soon.", "https://aur.archlinux.org/ currently serves the read-only web interface, which is consistent with pushes being disabled rather than the whole service being offline.", "Hacker News discussion: https://news.ycombinator.com/item?id=49146238"]
  reason: "AUR Git pushes are intentionally disabled by the Arch Linux DevOps team while it handles malicious package adoptions and follow-up commits. The public AUR web interface is online, but SSH push/clone for package publication returns the maintenance shutdown message. Retry only after Arch announces that pushes are enabled."
  updatedAt: "2026-08-03T21:29:47Z"
assignee: "orchestrator"
---

## Description

Prepare, validate, publish, and smoke-test Tandem 0.8.3. Update package version and curated release notes, run the canonical `just release 0.8.3` workflow, verify GitHub assets and AUR automation, and test the branded installer.
