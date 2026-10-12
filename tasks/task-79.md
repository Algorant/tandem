---
id: task-79
uid: 9fbf77e6-9cc2-45c9-8a2e-9d1fd0ca77b1
type: task
title: "Release Tandem 0.17.0"
state: "in-progress"
priority: "high"
effort: "medium"
references: ["task-69", "task-73", "task-74", "task-78"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
tags: ["config", "release"]
accord:
  status: "claimed"
  acceptance: ["Version and curated 0.17.0 notes cover session boards/full-TUI switching, local chooser activity, external-dependency guidance, optional archive notes and approved logo/site changes.", "Project release checks pass; main and annotated tandem-v0.17.0 tag are pushed and the matching non-draft GitHub Release publishes all expected binaries, installer and checksums.", "Published release assets and primary installer are verified; AUR workflow/package outcome and publication time are reported to Herdra."]
  claimedAt: "2026-10-12T20:28:26Z"
  validation: ["$ just release 0.17.0"]
  updatedAt: "2026-10-12T20:28:26Z"
createdAt: "2026-10-12T20:26:57Z"
updatedAt: "2026-10-12T20:28:26Z"
assignee: "tandem-ceph"
---

## Description

Algorant explicitly authorized task-74 integration, push and release 0.17.0 via Herdra. Direct orchestrator release work after verifying delivered task-74 and following tandem/RELEASE.md and just release. Protocol stays 0.5.0; no migration. Docs-overhaul/task-76 remain paused and unmerged. Report tag, release URL and confirmed asset-publication time so Herdra can update desktop/x1nano with mise. AUR workflow runs normally; record its outcome separately if downstream publication fails.
