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
  status: "claimed"
  acceptance: ["When x1nano is reachable, its existing mise-managed latest install selects the latest verified Tandem release, with the PATH command and actual selected executable version and SHA-256 checked against the published release artifact.", "No core or TUI on x1nano is restarted while updating it."]
  claimedAt: "2026-11-10T17:55:18Z"
  updatedAt: "2026-11-10T17:55:18Z"
createdAt: "2026-11-10T16:00:02Z"
updatedAt: "2026-11-10T17:55:40Z"
assignee: "tandem"
---

## Description

Standing reconnect task so each release does not add another x1nano card (Herdra's alternating-machine queue relies on the needs-x1nano tag).

Current target: 0.17.6 (release verified 2026-11-10; desktop and homelab updated to 0.17.6 with binary SHA-256 52192b8c…). x1nano was reachable for 0.17.5 and now times out; it still runs 0.17.5.

On reconnect: `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem`, then check the PATH command and `mise which tandem` actual executable report the then-current verified latest version, and compare its SHA-256 with the release archive. Update this Task body for later releases instead of filing new cards; complete only if the machine is retired. No new install path, pin, core/TUI restart or config change.
