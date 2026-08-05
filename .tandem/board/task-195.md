---
id: task-195
type: task
title: "Release Tandem 0.8.3"
state: "in-progress"
priority: "high"
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/RELEASE.md"]
tags: ["release"]
createdAt: "2026-08-03T21:11:10Z"
updatedAt: "2026-08-05T18:11:57Z"
accord:
  status: "blocked"
  assignee: "orchestrator"
  claimedAt: "2026-08-03T21:11:14Z"
  evidence: ["0.8.4 GitHub Release workflow 31033177672 succeeded and the branded installer installed tandem 0.8.4.", "0.8.4 AUR workflow 31033480566 failed at SSH clone with the same maintenance shutdown message as 0.8.3.", "https://aur.archlinux.org/packages/tandem-bin reports version 0.8.1-1 and last update 2026-07-31."]
  note: "Do not mark 0.8.3 successful yet. A successful 0.8.4 AUR publication can resolve this superseded release task because AUR only needs the latest package version."
  reason: "The 0.8.4 retry confirms that the 0.8.3 release blocker remains external AUR maintenance. GitHub publishing and installers work, but AUR SSH Git access is still disabled and tandem-bin remains at 0.8.1-1."
  updatedAt: "2026-08-05T18:11:57Z"
assignee: "orchestrator"
---

## Description

Prepare, validate, publish, and smoke-test Tandem 0.8.3. Update package version and curated release notes, run the canonical `just release 0.8.3` workflow, verify GitHub assets and AUR automation, and test the branded installer.
