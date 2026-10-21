---
id: task-89
uid: e93cc60f-cc02-4899-a718-91d800463cfb
type: task
title: "Release Tandem 0.17.3 and upgrade online clients together"
priority: "high"
effort: "medium"
references: ["task-53", "task-86", "task-87", "task-90"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "site/bun.lock", "justfile", "scripts/release_checks.sh"]
tags: ["config", "release"]
accord:
  status: "accepted"
  acceptance: ["Verified task53 implementation and actual Rules On/Off/count/toggle screenshot have passed Herdra/Algorant's pre-release gate; source history retains task87 and task86.", "Cargo version/lock and exactly one curated0.17.3 RELEASES section describe Added starters/on-off and Fixed idle background Git churn; maintained justrelease gates pass.", "Annotated tandem-v0.17.3 tag and non-draft GitHub Release include curated notes, required assets and verified checksums; isolated live-site installer reports0.17.3, and downstream AUR outcome is recorded.", "Desktop and archbox actual resolved binaries report0.17.3 and open Tandem TUIs are restarted/verified; offline x1nano catch-up is explicitly tracked without claiming it upgraded."]
  claimedAt: "2026-10-21T02:22:55Z"
  deliveredAt: "2026-10-21T02:36:41Z"
  validation: ["$ just release 0.17.3"]
  constraints: ["Authorized by Algorant via Herdra mecd47994/m084bd177; release and push are allowed only after task53 verification and actual TUI screenshot review gate.", "Preserve published 0.17.2/task87; ship task86 idle-I/O guard plus task53 starters/on-off as0.17.3. No unrelated release machinery or adapter changes.", "Upgrade desktop and archbox together immediately after verified publication; x1nano when back online. Restart only Tandem TUIs after preserving their cwd/arguments/session boards; do not interrupt agents or other services."]
  summary: "Released0.17.3 at66a87b0 after Algorant approved actualUI; task53accepted/integrated9fdfb18, task86/task87 preserved. All maintained gates passed after authorized Vite lock-only security refresh.13assets/checksums and liveinstaller smoke verified. Desktop+archbox upgraded together to0.17.3; noTUIsrunning. x1nanooffline trackedtask90."
  evidence: ["Annotated tag tandem-v0.17.3 targets66a87b0016dcb47621991a8f70176be0189435a8; main clean/synced. GitHubRelease non-draft/non-prerelease: https://github.com/Algorant/tandem/releases/tag/tandem-v0.17.3. Release workflow39533028629 and docs39533027084 successful; curated Added/Fixed notes included.", "Initial prepublication highaudit failed for Vite8.1.0/GHSA-587h-7wmj-h5pm; Herdra explicitly authorized existing-range8.3.4 lock-only refresh. Astro/packagejson unchanged; build,176links, highaudit and unchanged justrelease passed; no bypass/machinery changes. Release retry exit0.", "All13release assets downloaded; aggregate and every per-archive checksum verified. Linux archive SHA256 eaf517f376d16051f43a146fb2bb83a1863543d52ac989877ba04b34f5214ad7. Live trytandem.dev/install.sh matched released installer byte-for-byte, SHA256 ca48ade357bb6f4d81270e4aa63d3cc5160fdd00928663b20c2a1fa9ad77050f.", "Isolated liveinstaller installed0.17.3 to its reported installed/bin path with temporaryHOME/noPATHmodification. Fresh init6enabledstarters, noabsentagentfilescreated, disable→default5/all6→sameIDenable6passed. Installer and desktop/archbox mise binary all share SHA2561eda613fac8151c594c62e9b2b10c30ab30482ec525c8abdd346bc48f38c19fc.", "AUR workflow39533413367 success. PublicAURGit HEADfa3295a6b440370719b29c9c1a91ab44b70fe945 contains0.17.3-1 and matchingLinuxchecksum; RPC metadata still0.17.2-1 at finalcheck (cache lag, not package publication failure).", "Desktop and archbox concurrently upgraded via existingmise GitHub backend with observed versions-host bypass env; actual resolved latest binaries report0.17.3. NoTandemprocesses on eitherhost before/after, so noTUIs required restart; no otherprocesses touched. x1nano confirmedoffline; futurecatch-up recordedtask90, not falsely markedupgraded.", "Worker task53presentation/worktree removed byworker_finish; safepreviewroute reset; ownedbrowsers/previewpanes/webservers stopped. Disposable installer/download/AUR/smoke fixtures removed; retained source/PNG and release/QA logs only."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "site/bun.lock"]
  reviewer: "Algorant"
  updatedAt: "2026-10-21T02:36:42Z"
createdAt: "2026-10-20T22:50:25Z"
updatedAt: "2026-10-21T02:36:42Z"
assignee: "tandem"
archivedAt: "2026-10-21T02:36:42Z"
resolution:
  outcome: "completed"
  reviewer: "Algorant"
---

## Description

Algorant via Herdra authorized combined0.17.3 release and coordinated upgrades. Parent-reconciled main21be411 preserves0.17.2 commit4c3f386 and task86 rebased21be411, pushed normally. Wait for task53 full implementation/native/render verification and send actual Rules screenshot to Herdra before release. Owner performs Cargo/lock version prep, concise Added/Fixed notes, maintained release workflow plus isolated installer/assets checks; then coordinated mise-owned desktop/archbox upgrades and TUI restarts, x1nano when online. Older readers ignore enabled=false, so coordinated client adoption is required before relying on disabled rules. Task88 captures versions-host lag; don't silently claim an old resolved binary upgraded.
