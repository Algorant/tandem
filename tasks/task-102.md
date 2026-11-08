---
id: task-102
uid: 7b891c16-1665-49dc-8891-286fdb7b4ce0
type: task
title: "Preserve and restart only the two old Board clients after Algorant approves"
state: "validation"
priority: "medium"
effort: "medium"
references: ["task-100"]
tags: ["tui", "plan-first"]
accord:
  status: "claimed"
  acceptance: ["Algorant approves the narrow two-client restart plan and the homelab original terminal/view/draft is confirmed before any restart.", "Only the two identified Board clients are replaced with the verified installed latest binary, preserving each workspace roster and visible view/selection/filters without losing unsaved input.", "Actual replacement process versions and Board reads are verified; no core restart, unrelated process action, seedbox interruption, protocol migration or manual board-record edits occur."]
  claimedAt: "2026-11-08T05:26:38Z"
  updatedAt: "2026-11-08T05:26:38Z"
createdAt: "2026-11-08T05:26:13Z"
updatedAt: "2026-11-08T05:27:14Z"
assignee: "tandem"
validation.criterion: "Algorant approves the narrow two-client restart plan and the homelab original terminal/view/draft is confirmed before any restart."
validation.note: "Narrow plan recorded in the Task; ready for Herdra to present to Algorant. Nothing restarted. Desktop native HC target is verified; homelab is a distinct legacy PTY/pane, not HCcart-lab/t1, so its exact accessibility and view/draft must be confirmed before operation."
validation.requestedAt: "2026-11-08T05:26:39Z"
validation.reviewer: "Algorant"
---

## Description

Plan requested by Herdra mfe005aa9 after accepted 0.17.5 release. No restart authorized yet.

## Exact targets / before → after
- Desktop Herdra Board: native HC terminal t1v (`herdra / tandem`), PID2339450, cwd `~/Projects/herdra`, currently0.17.4 → fresh installed latest0.17.5 in the same terminal/cwd.
- Homelab legacy Board: PID463753, parent oldzsh1413, tty `/dev/pts/0`, original legacy pane identity `w6535cfd45e6c11:p17` (read from whitelisted process metadata). Runs deleted0.17.3 in `~/cart-lab` → fresh installed latest0.17.5 in that SAME original terminal/cwd. It is NOT current HCcart-lab/t1. No use of retired Herdr tooling as a fallback; cart-lab's Ceph/Algorant must confirm the exact original terminal is accessible before proceeding.

## Preservation and execution (only after approval)
1. Revalidate target PID/starttime, tty, cwd, actual executable and argv. Capture visible workspace/view/subview, selected Task ID/scroll anchor, filters, sort/arrangement, expanded rows/detail pane and ephemeral workspace roster. No session-state export exists, so preserve via observation and existing controls, not an invented saved-state file. If drafts/editor prompts exist, stop and have Algorant finish/save them; never send quit blindly.
2. Keep a temporary backup/hash of each actual running executable (`/proc/PID/exe`) and original argv before quitting, including the deleted homelab binary, so recovery is concrete. No install/config/board-file changes.
3. Quit ONLY each identified TUI gracefully in its original terminal, one at a time; allow normal final sync, and do not force-kill a slow exit. Reopen `tandem tui` with original startup arguments/session workspace roster and cwd using the installed verified latest executable. Homelab original terminal is user-operated if it has no native HC route; do not substitute another tab.
4. Restore captured session view using existing TUI controls. Saved workspace config stays untouched; session-cleared filters must remain cleared. If exact roster/view cannot be reconstructed, stop before quitting the old client and resolve that preservation gap first.

## Verification and recovery
Verify replacement PID's real executable/version (not just installed --version), same board/project, selected Task/filters/roster/view and prompt-free rendering, plus a bounded native Board read. Persistent task data must remain semantically unchanged except normal concurrent board updates/sync; no archive, migration, or manual board edits. No core restart, other terminal/shell termination or seedbox interruption.
If replacement startup fails, stop only the new client and report the error; with the approved recovery scope, reopen the backed-up previous executable/argv in the same cwd, retaining its known timeout vulnerability. Do not roll back board records or silently change runtime/version. Delete temporary executable backups only once the replacement checks pass.

Current read-only desktop snapshot: Board / ALL, hiding #work, selected task-48; recheck just before execution because live state can change. Homelab view/draft is not yet attested; confirmation requested from cart-lab's Ceph (m18dbb215).


## HOLD / target-use gate
Herdra m474dee4d holds BOTH restarts before asking Algorant. Legacy process existence is not a reason to revive retired infrastructure or relaunch it. Obtain cart-lab's read-only accessibility/current-use evidence, then return one short concrete choice: restart in an existing supported terminal, leave it running, or retire it with Algorant's approval. No restart, keys, process termination or retired-tool fallback authorized. If owner intent/current view cannot be attested, record that explicitly rather than infer it.