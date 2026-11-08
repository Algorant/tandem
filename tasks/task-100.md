---
id: task-100
uid: 4cbe2ac7-db1d-412e-9ca6-900d0bbeee49
type: task
title: "Release 0.17.5 and verify installed binaries without restarting live sessions"
state: "in-progress"
priority: "high"
effort: "medium"
references: ["task-98", "task-99"]
accord:
  status: "claimed"
  acceptance: ["Annotated tandem-v0.17.5 tag and GitHub Release with curated notes, required assets and checksums are verified through the normal release gates.", "Available machines use verified 0.17.5 binaries through their existing latest install paths; unreachable machines are reported.", "Remaining old running Tandem sessions are identified, with no live TUI/core restart or seedbox interruption."]
  claimedAt: "2026-11-08T05:09:15Z"
  updatedAt: "2026-11-08T05:09:15Z"
createdAt: "2026-11-08T05:09:14Z"
updatedAt: "2026-11-08T05:09:15Z"
assignee: "tandem"
---

## Description

Explicit Algorant approval relayed by Herdra mf6b5b402: Release and update. Include completed papercut fixes and the Git timeout descendant cleanup. Publish the patch using the normal release gates; upgrade installed binaries on available desktop, homelab and laptop machines through existing owner-managed mise paths. Do not restart cores/TUIs or interrupt seedbox work. Incident task-98 stays open until cart-lab confirms one actual Northstar checkpoint succeeds.
