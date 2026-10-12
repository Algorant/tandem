---
id: task-79
uid: 9fbf77e6-9cc2-45c9-8a2e-9d1fd0ca77b1
type: task
title: "Release Tandem 0.17.0"
priority: "high"
effort: "medium"
references: ["task-69", "task-73", "task-74", "task-78"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
tags: ["config", "release"]
accord:
  status: "accepted"
  acceptance: ["Version and curated 0.17.0 notes cover session boards/full-TUI switching, local chooser activity, external-dependency guidance, optional archive notes and approved logo/site changes.", "Project release checks pass; main and annotated tandem-v0.17.0 tag are pushed and the matching non-draft GitHub Release publishes all expected binaries, installer and checksums.", "Published release assets and primary installer are verified; AUR workflow/package outcome and publication time are reported to Herdra."]
  claimedAt: "2026-10-12T20:28:26Z"
  deliveredAt: "2026-10-12T20:40:50Z"
  validation: ["$ just release 0.17.0"]
  summary: "Released Tandem 0.17.0 from 1f8b59a after accepted task-74 integration. Main and annotated tandem-v0.17.0 pushed; GitHub Release and all 13 assets verified, including aggregate/per-artifact checksums and isolated primary-installer smoke. AUR workflow successfully published Git package 0.17.0-1; public RPC still reports cached 0.16.3-1. Docs-overhaul remains paused/unmerged."
  evidence: ["just release 0.17.0 succeeded using pinned cargo-dist 0.32.0 via mise exec; project Rust release tests/build/dist/clippy, site build/security audit, JS syntax, curated notes and GitHub-publication gates passed.", "Tag tandem-v0.17.0 points to pushed main 1f8b59a5edf38feb017611e1d468a04f39a9f526; annotated tag object 8475dfad38dac23de85dc0f935b9ff107311c12e. Main clean and remote-matched.", "https://github.com/Algorant/tandem/releases/tag/tandem-v0.17.0: non-draft/non-prerelease; all 13 non-empty assets uploaded by 20:36:16 UTC and release published 2026-10-12T20:36:17Z. Release workflow 38304625137 succeeded.", "Downloaded all assets; aggregate checksums and every per-artifact .sha256 check passed. Primary https://trytandem.dev/install.sh returned HTTP 200, matched published installer bytes and API SHA256 digest; isolated install ran tandem 0.17.0 and tui --help confirmed repeatable session-only --add-workspace.", "AUR workflow 38305322702 succeeded at 2026-10-12T20:37:19Z, pushing commit 5807915. Fresh AUR Git clone PKGBUILD/.SRCINFO specifies 0.17.0-1 with verified x86_64 checksum 5de07883ee4e909ec4a6955ffb5b577e4a296e585ff3f83fb73fd05aff6e61c0. RPC/web metadata remains stale at 0.16.3-1; this is not a failed Git publication.", "Herdra notified of exact asset-publication moment and verified availability for desktop/x1nano mise updates. No docs-overhaul branch integration or deployment; task-76 remains paused."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
  reviewer: "Algorant"
  updatedAt: "2026-10-12T20:40:52Z"
createdAt: "2026-10-12T20:26:57Z"
updatedAt: "2026-10-12T20:40:52Z"
assignee: "tandem-ceph"
archivedAt: "2026-10-12T20:40:52Z"
resolution:
  outcome: "completed"
  reviewer: "Algorant"
---

## Description

Algorant explicitly authorized task-74 integration, push and release 0.17.0 via Herdra. Direct orchestrator release work after verifying delivered task-74 and following tandem/RELEASE.md and just release. Protocol stays 0.5.0; no migration. Docs-overhaul/task-76 remain paused and unmerged. Report tag, release URL and confirmed asset-publication time so Herdra can update desktop/x1nano with mise. AUR workflow runs normally; record its outcome separately if downstream publication fails.
