---
id: task-105
uid: de0b8e22-a899-42bf-bf69-b7c41f608f3c
type: task
title: "Keep x1nano's installed Tandem on the latest verified release"
state: "in-progress"
priority: "low"
effort: "small"
tags: ["config"]
accord:
  status: "delivered"
  acceptance: ["When x1nano is reachable, its existing mise-managed latest install selects the latest verified Tandem release, with the PATH command and actual selected executable version and SHA-256 checked against the published release artifact.", "No core or TUI on x1nano is restarted while updating it."]
  claimedAt: "2026-11-10T17:55:18Z"
  deliveredAt: "2026-11-10T17:55:42Z"
  summary: "x1nano reconnect update to0.17.6: existing mise latest install upgraded from0.17.5, PATH command and actual selected executable both verified0.17.6 with the release SHA-256; no core/TUI restarted."
  evidence: ["GitHub had verified stable tandem-v0.17.6 before update. `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem` on x1nano installed0.17.6 (2.5s) from the released linux archive.", "On x1nano: PATH shim `~/.local/share/mise/shims/tandem` and actual `mise which tandem` executable `.../github-algorant-tandem/0.17.6/tandem` both report tandem0.17.6; actual binary SHA256 52192b8cf2d00d264e153df6372ab2a2e405214e1e9b482620c43f9b1033fce7 matches the verified release and desktop/homelab installs.", "Native /proc snapshot found no Tandem TUI/web process on x1nano, so nothing was restarted or reloaded. needs-x1nano tag removed before completion."]
  updatedAt: "2026-11-10T17:55:42Z"
createdAt: "2026-11-10T16:00:02Z"
updatedAt: "2026-11-10T17:55:42Z"
assignee: "tandem"
---

## Description

Standing reconnect task so each release does not add another x1nano card (Herdra's alternating-machine queue relies on the needs-x1nano tag).

Current target: 0.17.6 (release verified 2026-11-10; desktop and homelab updated to 0.17.6 with binary SHA-256 52192b8c…). x1nano was reachable for 0.17.5 and now times out; it still runs 0.17.5.

On reconnect: `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem`, then check the PATH command and `mise which tandem` actual executable report the then-current verified latest version, and compare its SHA-256 with the release archive. Update this Task body for later releases instead of filing new cards; complete only if the machine is retired. No new install path, pin, core/TUI restart or config change.
