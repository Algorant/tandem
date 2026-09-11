---
id: task-1
type: task
title: "AUR tandem-bin needs a 0.12.0 build"
priority: "low"
tags: ["papercut"]
accord:
  status: "accepted"
  acceptance: ["tandem-bin AUR package updated to 0.12.0 once the AUR accepts pushes"]
  claimedAt: "2026-09-11T14:22:41Z"
  deliveredAt: "2026-09-11T14:23:03Z"
  summary: "Resolved by later publication: tandem-bin 0.13.1-1 is available in AUR. Algorant approved closing this stale 0.12.0 availability papercut rather than backporting an obsolete package version."
  evidence: ["Read-only AUR RPC https://aur.archlinux.org/rpc/v5/info?arg[]=tandem-bin returned Version=0.13.1-1 and OutOfDate=null during the audit.", "GitHub AUR run 34518338245 for tandem-v0.13.1 completed successfully including Commit and push AUR update; the historical read-only blocker no longer prevents publication.", "Task-31 records primary release, installer and checksum verification; task-27 will audit outdated AUR read-only assumptions in release guidance."]
  updatedAt: "2026-09-11T14:23:14Z"
createdAt: "2026-08-31T20:53:43Z"
updatedAt: "2026-09-11T14:23:14Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-11T14:23:14Z"
resolution:
  outcome: "completed"
---

## Description

Rule 6: AUR verification is skipped while the AUR is read-only. Tracked as a downstream packaging issue; carried over from the archived workspace papercut-13.
