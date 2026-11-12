---
id: task-108
uid: d8e4ebcc-18c2-43db-97ff-6f96e0d8eef1
type: task
title: "Update x1nano's installed tandem to 0.17.7 when it reconnects"
state: "in-progress"
priority: "low"
effort: "small"
references: ["task-105", "task-107"]
tags: ["config"]
accord:
  status: "delivered"
  acceptance: ["On reconnect, x1nano's existing mise-managed latest install selects tandem 0.17.7 (or a later verified release), and the PATH command plus the actual selected executable report that version with its SHA-256 matching the published archive.", "No core or TUI on x1nano is restarted while updating it."]
  claimedAt: "2026-11-12T03:04:13Z"
  deliveredAt: "2026-11-12T03:04:32Z"
  summary: "x1nano reconnect update to0.17.7: existing mise latest install upgraded from0.17.6, PATH command and actual selected executable verified with the release SHA-256; no core/TUI restart."
  evidence: ["GitHub latest was stable tandem-v0.17.7. `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem` on x1nano installed0.17.7 from the released linux archive.", "On x1nano, the PATH shim and the actual `mise which tandem` executable both report tandem0.17.7; actual binary SHA256 6dc87bfd2200a52614b492f0206b2637d2d03a115d51b46832df9fa008cff0e5 matches the verified release and the desktop/homelab installs.", "Native /proc snapshot found no tandem TUI or web process on x1nano, so nothing was restarted or reloaded. needs-x1nano tag removed before completion."]
  updatedAt: "2026-11-12T03:04:32Z"
createdAt: "2026-11-11T18:33:55Z"
updatedAt: "2026-11-12T03:04:32Z"
assignee: "tandem"
---

## Description

Queued by Herdra m7017836b after the 0.17.7 release: x1nano was unreachable (SSH timeout), so it remains on 0.17.6 while desktop and homelab run 0.17.7.

Do: `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem`, then check the PATH command and `mise which tandem` executable both report 0.17.7 with SHA-256 6dc87bfd2200a52614b492f0206b2637d2d03a115d51b46832df9fa008cff0e5 (or the then-current verified release). No new install path, pin, core/TUI restart or config change. Remove the needs-x1nano tag or complete this Task when done; herdra-online notifies on reconnect.
