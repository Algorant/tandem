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
  status: "claimed"
  acceptance: ["Package and lockfile identify0.13.2 and concise curated release notes describe the six implemented papercut fixes.", "Required release checks pass; main and annotated tandem-v0.13.2 tag are pushed and a non-draft/non-prerelease GitHub Release contains curated notes, four platform archives, installer and checksums.", "All published checksums and the primary branded installer are verified; isolated installation reports tandem0.13.2 and passes relevant CLI smoke. Record actual non-blocking AUR result."]
  claimedAt: "2026-09-11T16:56:00Z"
  validation: ["$ just release 0.13.2", "Verify branded installer, isolated installed binary, release checksums and actual AUR workflow result."]
  updatedAt: "2026-09-11T16:56:00Z"
createdAt: "2026-09-11T16:55:50Z"
updatedAt: "2026-09-11T16:56:00Z"
assignee: "pi-orchestrator"
---

## Description

Algorant approved the completed papercut cleanup and explicitly requested committing, pushing and release0.13.2. Current package/latest release is0.13.1. Use the established just release path without changing release automation or unrelated backlog. Verify artifacts/checksums and the branded installer in disposable state; inspect actual AUR outcome rather than assuming the obsolete read-only limitation still applies.
