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
updatedAt: "2026-11-08T05:01:55Z"
assignee: "tandem"
references: ["task-99"]
---

## Description

Herdra incident request mc92d047b: on the homelab board, Tandem 0.17.4 `show task-105 --json` times out while a long-lived Tandem TUI owns `.git/tandem/lock`; readers are blocked in locks_lock_inode_wait. Coordinate all intervention with cart-lab's Ceph before acting; preserve its seedbox reclaim verification. Related checkpoint symptom: Pi board task-613.

Investigate lock lifetime and live process descendants read-only, establish safe recovery with the board owner, and identify the smallest robust prevention fix. No blind lock deletion, no restart/kill without coordination, no real homelab-board record writes. Implementation scope/plan must follow the established cause, not a speculative timeout/fallback framework.


## Established diagnosis
Installed CLI is 0.17.4, but the live lock holder is the deleted 0.17.3 TUI executable (PID463753). It has no direct children. Thread3072903 waits in anon_pipe_read; fd10 reads pipe186417764. Orphan SSH3072904 (PPID1, GitHub git-upload-pack for this repository) keeps that identical pipe open as stderr fd2. Thus EOF cannot reach the TUI's pipe-reader thread; sync retains its RAII hierarchy lock. Source `project/git.rs` kills/waits only for Git on timeout then unconditionally joins read_to_end reader threads, matching this live state. The initiating SSH/network stall itself was not traced.

## Recovery gate and prevention
Herdra approved a narrow intervention (m122e8c27), but no signal is sent until cart-lab's Ceph confirms a safe window for the active seedbox verification. Proposed recovery: revalidate the orphan PID/PPID/argv/pipe relationship and TERM only that SSH process, then verify flock release and bounded read-only `show`. No TUI restart, lock deletion or board-record mutation.

Prevention is separately owned by task-99 (Bound Git timeout Worker): plan-first shared Git runner subprocess-group cleanup, with native Git/SSH timeout E2E and post-timeout board read. Pi task-613 owns wrapper diagnostics separately.