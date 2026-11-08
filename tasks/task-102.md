---
id: task-102
uid: 7b891c16-1665-49dc-8891-286fdb7b4ce0
type: task
title: "Restart the supported desktop Board after preserving its current view"
state: "validation"
priority: "medium"
effort: "medium"
references: ["task-100"]
tags: ["tui", "plan-first"]
accord:
  status: "claimed"
  acceptance: ["Algorant approves the desktop same-terminal restart plan before any action; the legacy homelab Board remains untouched.", "The approved desktop restart uses the same supported terminal and verified installed latest binary, preserving its workspace roster and visible view/selection/filters without losing unsaved input.", "Actual desktop replacement process version and Board read are verified; no legacy-client action, core restart, unrelated process action, seedbox interruption, protocol migration or manual board-record edits occur."]
  claimedAt: "2026-11-08T05:26:38Z"
  updatedAt: "2026-11-08T06:29:41Z"
createdAt: "2026-11-08T05:26:13Z"
updatedAt: "2026-11-08T06:29:41Z"
assignee: "tandem"
validation.criterion: "Algorant approves the narrow two-client restart plan and the homelab original terminal/view/draft is confirmed before any restart."
validation.note: "Narrow plan recorded in the Task; ready for Herdra to present to Algorant. Nothing restarted. Desktop native HC target is verified; homelab is a distinct legacy PTY/pane, not HCcart-lab/t1, so its exact accessibility and view/draft must be confirmed before operation."
validation.requestedAt: "2026-11-08T05:26:39Z"
validation.reviewer: "Algorant"
---
## Authorization: HOLD both clients
Herdra m474dee4d holds both restarts pending evidence/Algorant's choice. No restart, termination, keys, reattach, core action or legacy-tool fallback authorized. The original proposal to relaunch both clients is superseded by this target/use gate.

## Verified targets
- Desktop Herdra Board: native HCterminal t1v (`herdra / tandem`), PID2339450, cwd `~/Projects/herdra`, running0.17.4; installed latest0.17.5. Read-only snapshot: Board/ALL, hiding#work, selectedtask-48 (recheck before any approved action).
- Homelab legacy Board: PID463753 → oldzsh1413 → retired Herdr0.9.5 daemon1402; controlling tty `/dev/pts/0`, original legacy pane(w6535cfd45e6c11:p17), tab(w6535cfd45e6c11:tB). Running deleted0.17.3, installed latest0.17.5. No HC_TERMINAL_ID/HC_SESSION or current native-space identity. NOT HCcart-lab/t1. Owner read-only evidence m9df75914 cannot attest current visibility, unsaved draft/roster, or human intent to keep it.

## Current recommendation
Leave the legacy client running for now, not because existence proves it is wanted, but because neither a supported restart target nor permission to discard its unobservable UI state exists. Retirement requires Algorant's explicit decision; do not revive retired infrastructure. If a new homelab Board is wanted later, plan it in an existing supported terminal instead of assuming this session should be relaunched. Both clients remain untouched meanwhile.

## Desktop restart plan, only after separate approval
Before0.17.4 → after verified installed latest0.17.5 in the SAME native Board terminal/cwd. Recheck PID/starttime/argv; capture live workspace roster, view/subview, selected Task/scroll anchor, filters/sort/arrangement and expansion/detail state. Stop for Algorant to finish/save any unsaved prompt/editor input; stop before quitting if view/roster restoration is uncertain. Retain original executable/argv for recovery. Gracefully quit ONLY that TUI, allow normal final sync, relaunch with same cwd/startup args and restore captured session view with existing controls. No force-kill, shell/core restart, config edits or manual board edits.

Verify actual replacement process executable/version (not installed --version alone), same board/roster/view/selection/filters, prompt-free rendering and a bounded native Board read. If replacement startup fails, stop only the new client and report; the approved recovery may reopen the retained prior executable/argv (known old timeout vulnerability). Never roll back board records. Remove temporary recovery copies only after successful checks.

## Final scope from Herdra mcf7a3a22
Leave the legacy homelab Board untouched. Its restart, retirement, reattach, further probes, or replacement are NOT part of this Task. Only the desktop supported-terminal plan is queued for Algorant's approval, with view preservation and actual running-version verification. No runtime action is approved yet.