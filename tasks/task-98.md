---
id: task-98
uid: d26c5994-71f7-462a-9457-dd595ba83cf4
type: task
title: "Diagnose and recover a stuck TUI board-sync lock without disrupting active work"
state: "in-progress"
priority: "high"
effort: "medium"
tags: ["tui", "sync"]
accord:
  status: "claimed"
  acceptance: ["Identify the process/control-flow cause holding the board lock with live evidence and source locations.", "Agree safe recovery with cart-lab's Ceph, preserve seedbox work, and verify read-only Board access returns promptly after recovery (or record why intervention cannot yet proceed).", "Record the smallest prevention fix and its real-system verification plan, with implementation delegated only once cause and scope are established."]
  claimedAt: "2026-11-08T04:59:57Z"
  updatedAt: "2026-11-08T04:59:57Z"
createdAt: "2026-11-08T04:59:47Z"
updatedAt: "2026-11-08T04:59:57Z"
assignee: "tandem"
---

## Description

Herdra incident request mc92d047b: on the homelab board, Tandem 0.17.4 `show task-105 --json` times out while a long-lived Tandem TUI owns `.git/tandem/lock`; readers are blocked in locks_lock_inode_wait. Coordinate all intervention with cart-lab's Ceph before acting; preserve its seedbox reclaim verification. Related checkpoint symptom: Pi board task-613.

Investigate lock lifetime and live process descendants read-only, establish safe recovery with the board owner, and identify the smallest robust prevention fix. No blind lock deletion, no restart/kill without coordination, no real homelab-board record writes. Implementation scope/plan must follow the established cause, not a speculative timeout/fallback framework.
