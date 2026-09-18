---
id: task-41
type: task
title: "Release Tandem 0.13.6"
state: "in-progress"
priority: "high"
effort: "medium"
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
tags: ["config", "validation"]
accord:
  status: "claimed"
  acceptance: ["Cargo package and lockfile identify 0.13.6, and concise curated notes accurately describe immediate metadata persistence, removal of automatic history rewriting, the forward-only checkpoint flush, and the outstanding host-adapter boundary wiring prerequisite.", "The release preparation is committed on main; `just release 0.13.6` passes, pushes main and annotated tag `tandem-v0.13.6`, and verifies a non-draft/non-prerelease GitHub Release with curated notes, required platform archives, installer, and checksums.", "The primary installer is smoke-tested and reports `tandem 0.13.6`; the temporary AUR verification skip is recorded as a non-blocking downstream packaging limitation."]
  claimedAt: "2026-09-18T21:50:03Z"
  validation: ["$ just release 0.13.6", "$ curl -fsSL https://trytandem.dev/install.sh | sh && ~/.local/bin/tandem --version"]
  constraints: ["Do not modify Pi adapters, Worktrunk, or the paused task-201 checkout.", "Do not reuse or move an existing release tag; publish 0.13.6 as the next patch after 0.13.5.", "Do not push if Tandem checkpointing fails or owning `.tandem/` remains dirty."]
  updatedAt: "2026-09-18T21:50:03Z"
createdAt: "2026-09-18T21:50:00Z"
updatedAt: "2026-09-18T21:50:03Z"
assignee: "pi"
---

## Description

Prepare and publish the patch release containing task-40's forward-only metadata batching fix. Curate concise release notes, update package/lock versions, run the repository release recipe, verify the annotated tag and GitHub Release assets/notes, and smoke-test the primary installer. Record the temporary AUR verification skip as a downstream packaging limitation. Do not include Pi adapter implementation or repair the paused task-201 rebase in this release Task.
