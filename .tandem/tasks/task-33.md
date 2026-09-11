---
id: task-33
type: task
title: "Publish Tandem 0.13.2 papercut-fixes release"
state: "in-progress"
effort: "medium"
references: ["task-2", "task-16", "task-17", "task-21", "task-28", "task-32", "decision-4", "task-27"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
tags: ["docs", "release"]
accord:
  status: "delivered"
  acceptance: ["Package and lockfile identify0.13.2 and concise curated release notes describe the six implemented papercut fixes.", "Required release checks pass; main and annotated tandem-v0.13.2 tag are pushed and a non-draft/non-prerelease GitHub Release contains curated notes, four platform archives, installer and checksums.", "All published checksums and the primary branded installer are verified; isolated installation reports tandem0.13.2 and passes relevant CLI smoke. Record actual non-blocking AUR result."]
  claimedAt: "2026-09-11T16:56:00Z"
  deliveredAt: "2026-09-11T17:04:41Z"
  validation: ["$ just release 0.13.2", "Verify branded installer, isolated installed binary, release checksums and actual AUR workflow result."]
  summary: "Published Tandem0.13.2 with the six Algorant-approved papercut fixes. Main and annotated tag are pushed; GitHub Release, all artifacts/checksums, primary installer and actual AUR publication are verified."
  evidence: ["just release0.13.2 exited0: formatting, release tests, release/dist builds, strict Clippy, docs build/security audit, web syntax and project adapter smoke tests passed; Release workflow completed and the recipe verified curated notes and all required nonempty assets.", "Annotated tandem-v0.13.2 points to83551dc1b032b2feded805452bf5592f8ee5016e, the pushed release commit. https://github.com/Algorant/tandem/releases/tag/tandem-v0.13.2 is published, not draft or prerelease, with all four platform archives, installer and aggregate/per-archive checksums.", "/tmp/tandem-release-0.13.2.zmAaeP/verify.sh exited0: every aggregate and per-archive SHA-256 checksum passed; https://trytandem.dev/install.sh was byte-identical to the published installer and declared APP_VERSION0.13.2.", "The branded installer installed into isolated /tmp HOME without changing user PATH/profiles. Downloaded Linux binary reported tandem0.13.2 and passed independent native init/review, Decision metadata/date/no-op, and nested Epic closure outcome probes.", "AUR workflow34625392046 completed successfully. Fresh AUR RPC reports tandem-bin Version0.13.2-1 and OutOfDate=null. The recipe's obsolete read-only wording remains tracked under task-27; it was not treated as evidence of current AUR failure."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
  updatedAt: "2026-09-11T17:04:41Z"
createdAt: "2026-09-11T16:55:50Z"
updatedAt: "2026-09-11T17:04:41Z"
assignee: "pi-orchestrator"
---

## Description

Algorant approved the completed papercut cleanup and explicitly requested committing, pushing and release0.13.2. Current package/latest release is0.13.1. Use the established just release path without changing release automation or unrelated backlog. Verify artifacts/checksums and the branded installer in disposable state; inspect actual AUR outcome rather than assuming the obsolete read-only limitation still applies.
