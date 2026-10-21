---
id: task-89
uid: e93cc60f-cc02-4899-a718-91d800463cfb
type: task
title: "Release Tandem 0.17.3 and upgrade online clients together"
state: todo
priority: "high"
effort: "medium"
references: ["task-86", "task-87"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "justfile", "scripts/release_checks.sh"]
tags: ["config", "release"]
accord:
  status: "ready"
  acceptance: ["Verified task53 implementation and actual Rules On/Off/count/toggle screenshot have passed Herdra/Algorant's pre-release gate; source history retains task87 and task86.", "Cargo version/lock and exactly one curated0.17.3 RELEASES section describe Added starters/on-off and Fixed idle background Git churn; maintained justrelease gates pass.", "Annotated tandem-v0.17.3 tag and non-draft GitHub Release include curated notes, required assets and verified checksums; isolated live-site installer reports0.17.3, and downstream AUR outcome is recorded.", "Desktop and archbox actual resolved binaries report0.17.3 and open Tandem TUIs are restarted/verified; offline x1nano catch-up is explicitly tracked without claiming it upgraded."]
  validation: ["$ just release 0.17.3"]
  constraints: ["Authorized by Algorant via Herdra mecd47994/m084bd177; release and push are allowed only after task53 verification and actual TUI screenshot review gate.", "Preserve published 0.17.2/task87; ship task86 idle-I/O guard plus task53 starters/on-off as0.17.3. No unrelated release machinery or adapter changes.", "Upgrade desktop and archbox together immediately after verified publication; x1nano when back online. Restart only Tandem TUIs after preserving their cwd/arguments/session boards; do not interrupt agents or other services."]
  updatedAt: "2026-10-20T22:50:25Z"
createdAt: "2026-10-20T22:50:25Z"
updatedAt: "2026-10-21T02:22:29Z"
---

## Description

Algorant via Herdra authorized combined0.17.3 release and coordinated upgrades. Parent-reconciled main21be411 preserves0.17.2 commit4c3f386 and task86 rebased21be411, pushed normally. Wait for task53 full implementation/native/render verification and send actual Rules screenshot to Herdra before release. Owner performs Cargo/lock version prep, concise Added/Fixed notes, maintained release workflow plus isolated installer/assets checks; then coordinated mise-owned desktop/archbox upgrades and TUI restarts, x1nano when online. Older readers ignore enabled=false, so coordinated client adoption is required before relying on disabled rules. Task88 captures versions-host lag; don't silently claim an old resolved binary upgraded.
