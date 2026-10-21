---
id: task-90
uid: 8ee67079-7198-46b0-84b1-f61447d6c088
type: task
title: "Upgrade x1nano to the coordinated Tandem release when online"
priority: "medium"
effort: "small"
references: ["task-89"]
tags: ["config", "upgrade"]
accord:
  status: "accepted"
  acceptance: ["After x1nano is online, its actual mise-resolved Tandem binary reports the coordinated release (at least0.17.3) before resuming shared-board work.", "Any old open Tandem TUIs are restarted with cwd/arguments/session-board entries preserved; no agents or unrelated services are interrupted."]
  claimedAt: "2026-10-21T03:28:14Z"
  deliveredAt: "2026-10-21T03:32:08Z"
  constraints: ["External availability condition: x1nano was offline at0.17.3 publication; do not claim it upgraded or repeatedly probe/wait.", "Use existing mise/SSH workflow; no source implementation, new infrastructure or profile changes."]
  summary: "x1nano upgraded from0.17.1 to0.17.3; resolved and running-TUI executable hashes match the published release. Restarted exactly one Tandem TUI, preserving Algomarchy cwd, tui arguments, sole session board and original Logs/task90selection. Otherpanes/agents untouched; SSH stayedonline."
  evidence: ["Tailscale reportedx1nanoonline and boundedSSH remainedreachable throughout; existing miseGitHub backend upgraded onlyTandem to0.17.3 using observed versions-host bypass env.", "Resolved misebinary and /proc/529127/exe both reporttandem0.17.3, SHA2561eda613fac8151c594c62e9b2b10c30ab30482ec525c8abdd346bc48f38c19fc, matching publishedbinary and desktop/archbox.", "Inventoried onlyoldTUI PID27936 in x1nano Herdrdefaultpane w1:pM, cwd/home/algorant/Projects/Algomarchy, argv tandem tui, sessionchooseroneAlgomarchyboard. Gracefully quit to its originalzsh1498, then restarted verifiedabsolute misebinary; newPID529127. SeparateEsc thenq was required (combinedEsc+q had notquit); no forcekill/otherprocesscontrol.", "NativeANSI afterrestart verified Board6/Logs128/Rules1/Decisions2, originalLogsview with selectedtask90detail restored, same singleAlgomarchy sessionboard. Closedchooser afterward. Finalpslisted onlynewTUI; otheragentpanes untouched; no source/config/profile/infrastructurechanges."]
  reviewer: "Algorant"
  updatedAt: "2026-10-21T03:32:09Z"
createdAt: "2026-10-21T02:35:32Z"
updatedAt: "2026-10-21T03:32:09Z"
assignee: "tandem"
archivedAt: "2026-10-21T03:32:09Z"
resolution:
  outcome: "completed"
  reviewer: "Algorant"
---

## Description

Algorant via Herdra authorized x1nano catch-up when back online as part of0.17.3 rollout. Desktop and archbox were upgraded together to0.17.3; x1nano stilloffline in Tailscale pre/postpublication checks. Older clients ignore enabled:false, so upgrade before shared-board work. This is explicit deferred tracking, not a claimed/started upgrade.
