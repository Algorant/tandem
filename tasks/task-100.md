---
id: task-100
uid: 4cbe2ac7-db1d-412e-9ca6-900d0bbeee49
type: task
title: "Release 0.17.5 and verify installed binaries without restarting live sessions"
state: "in-progress"
priority: "high"
effort: "medium"
references: ["task-98", "task-99"]
accord:
  status: "delivered"
  acceptance: ["Annotated tandem-v0.17.5 tag and GitHub Release with curated notes, required assets and checksums are verified through the normal release gates.", "Available machines use verified 0.17.5 binaries through their existing latest install paths; unreachable machines are reported.", "Remaining old running Tandem sessions are identified, with no live TUI/core restart or seedbox interruption."]
  claimedAt: "2026-11-08T05:09:15Z"
  deliveredAt: "2026-11-08T05:23:29Z"
  summary: "Published tandem-v0.17.5 and verified non-draft stable GitHub Release with all 13 required assets, aggregate/per-artifact checksums, curated notes and successful AUR workflow. Desktop and cart-lab installed binaries upgraded through existing mise latest paths and matched release SHA-256; x1nano unreachable and explicitly skipped. Two old running TUIs identified and deliberately not restarted."
  evidence: ["Release tag tandem-v0.17.5 points to121aee5 (annotated tag); main/source clean and synced. https://github.com/Algorant/tandem/releases/tag/tandem-v0.17.5 is non-draft/non-prerelease,13 nonempty assets. Normal just release gates and matching AUR workflow42450438802 passed.", "First attempt42449693697 failed before GitHub Release/assets because macos14 ARM runner retired Nov2. Minimal dist custom runner121aee5 uses macos-latest; dist plan confirms aarch64 host/target and generate --check passed. Original unpublished tag backed up in verified bundle;404Release confirmed before retry per existing procedure. No source-branch history rewrite or bypassed gate.", "Downloaded all release assets; aggregate sha256.sum and every per-archive checksum passed. Linux binary reports0.17.5. Live https://trytandem.dev/install.sh byte-matches release installer; isolated unmanaged smoke install reports0.17.5 and matches released binary SHA256 bfcd6248ca9fb6f270a42feb4d3757fc975a7c79aa49a64efda0c77a5f786bee.", "Desktop and cart-lab: existing MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem upgraded0.17.4→0.17.5. Actual selected executables (not just cart-lab's PATH shim) and command --version verified0.17.5; both executable hashes match released bfcd6248….", "x1nano HC connection unavailable: SSH connection timed out. No upgrade/restart attempted there; follow-up handed to Herdra.", "Actual /proc/<pid>/exe --version snapshot: desktop herdra TUI2339450 remains0.17.4; cart-lab TUI463753 remains deleted0.17.3. No other TUI/web process found on those checked machines. No core/TUI restart, seedbox interruption or visibility change. Incident98 actual Northstar recovery separately confirmed/completed."]
  updatedAt: "2026-11-08T05:23:29Z"
createdAt: "2026-11-08T05:09:14Z"
updatedAt: "2026-11-08T05:23:29Z"
assignee: "tandem"
---

## Description

Explicit Algorant approval relayed by Herdra mf6b5b402: Release and update. Include completed papercut fixes and the Git timeout descendant cleanup. Publish the patch using the normal release gates; upgrade installed binaries on available desktop, homelab and laptop machines through existing owner-managed mise paths. Do not restart cores/TUIs or interrupt seedbox work. Incident task-98 stays open until cart-lab confirms one actual Northstar checkpoint succeeds.
