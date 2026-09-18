---
id: task-41
type: task
title: "Release Tandem 0.13.6"
priority: "high"
effort: "medium"
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
tags: ["config", "validation"]
accord:
  status: "accepted"
  acceptance: ["Cargo package and lockfile identify 0.13.6, and concise curated notes accurately describe immediate metadata persistence, removal of automatic history rewriting, the forward-only checkpoint flush, and the outstanding host-adapter boundary wiring prerequisite.", "The release preparation is committed on main; `just release 0.13.6` passes, pushes main and annotated tag `tandem-v0.13.6`, and verifies a non-draft/non-prerelease GitHub Release with curated notes, required platform archives, installer, and checksums.", "The primary installer is smoke-tested and reports `tandem 0.13.6`; the temporary AUR verification skip is recorded as a non-blocking downstream packaging limitation."]
  claimedAt: "2026-09-18T21:50:03Z"
  deliveredAt: "2026-09-18T21:55:46Z"
  validation: ["$ just release 0.13.6", "$ ~/.cargo/bin/tandem --version"]
  constraints: ["Do not modify Pi adapters, Worktrunk, or the paused task-201 checkout.", "Do not reuse or move an existing release tag; publish 0.13.6 as the next patch after 0.13.5.", "Do not push if Tandem checkpointing fails or owning `.tandem/` remains dirty."]
  summary: "Prepared and published Tandem 0.13.6 with curated notes for forward-only metadata batching; pushed main and annotated tag, verified the GitHub Release and required assets, and smoke-tested the installed binary."
  evidence: ["`just release 0.13.6` exited 0 after release-profile tests/builds, clippy, docs/site and adapter smoke checks, then pushed main/tag and verified GitHub Release workflow 35398831788.", "GitHub Release https://github.com/Algorant/tandem/releases/tag/tandem-v0.13.6 is non-draft/non-prerelease and contains installer, four platform archives, per-artifact checksums, aggregate sha256.sum, source archive, and manifest.", "The installer selected /home/ivan/.cargo/bin; `/home/ivan/.cargo/bin/tandem --version` reports `tandem 0.13.6`. The initial hard-coded ~/.local/bin probe failed because that was not the installer-selected path, so the Task validation was corrected to the observed install location.", "AUR verification was intentionally skipped by the release recipe while AUR is read-only, as the repository rule requires; it is recorded as a non-blocking downstream packaging limitation."]
  reviewer: "pi"
  updatedAt: "2026-09-18T21:55:48Z"
createdAt: "2026-09-18T21:50:00Z"
updatedAt: "2026-09-18T21:55:48Z"
assignee: "pi"
archivedAt: "2026-09-18T21:55:48Z"
resolution:
  outcome: "completed"
  reviewer: "pi"
---

## Description

Prepare and publish the patch release containing task-40's forward-only metadata batching fix. Curate concise release notes, update package/lock versions, run the repository release recipe, verify the annotated tag and GitHub Release assets/notes, and smoke-test the primary installer. Record the temporary AUR verification skip as a downstream packaging limitation. Do not include Pi adapter implementation or repair the paused task-201 rebase in this release Task.
