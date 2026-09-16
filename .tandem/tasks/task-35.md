---
id: task-35
type: task
title: "Publish Tandem 0.13.3 native checkpoint-collapse release"
state: "in-progress"
priority: "high"
effort: "medium"
references: ["task-34", "decision-4"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
tags: ["docs"]
accord:
  status: "claimed"
  acceptance: ["Package and lockfile identify 0.13.3 and concise curated release notes describe native unpushed tandem-only checkpoint amend/reconcile, amended/consolidated JSON, and removal of just tidy-history.", "Required release checks pass; main and annotated tandem-v0.13.3 tag are pushed and a non-draft/non-prerelease GitHub Release contains curated notes, four platform archives, installer and checksums.", "All published checksums and the primary branded installer are verified; isolated installation reports tandem 0.13.3 and passes a relevant CLI smoke. Record the actual non-blocking AUR result."]
  claimedAt: "2026-09-16T17:43:07Z"
  validation: ["$ just release 0.13.3", "Verify branded installer, isolated installed binary, release checksums and actual AUR workflow result."]
  updatedAt: "2026-09-16T17:43:07Z"
createdAt: "2026-09-16T17:43:03Z"
updatedAt: "2026-09-16T17:43:07Z"
assignee: "pi-orchestrator"
---

## Description

Algorant asked for a clean checkout, commit/push of current main, then a small 0.13.3 release for native unpushed tandem-only checkpoint collapse. Use the established just release path without changing release automation or unrelated backlog. Verify artifacts/checksums and the branded installer in disposable state; inspect the actual AUR outcome.
