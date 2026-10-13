---
id: task-80
uid: f3fe8776-241c-46f2-bf61-b66f1a64a3db
type: task
kind: "papercut"
title: "Remove obsolete AUR read-only skip from release verification"
state: "in-progress"
priority: "low"
relatedFiles: ["justfile", "tandem/RELEASE.md"]
tags: ["config", "release"]
accord:
  status: "claimed"
  acceptance: ["Release recipe verifies the matching AUR workflow again and no longer claims AUR is read-only when publication succeeds.", "Release checklist removes obsolete read-only/skip instructions while preserving the rule that downstream AUR failure does not invalidate a verified GitHub release."]
  claimedAt: "2026-10-13T14:15:20Z"
  updatedAt: "2026-10-13T14:15:20Z"
createdAt: "2026-10-12T20:40:49Z"
updatedAt: "2026-10-13T14:15:20Z"
assignee: "tandem-ceph"
---

## Description

Observed while releasing 0.17.0: just release succeeded but printed SKIPPED AUR verification (AUR read-only), while matching workflow 38305322702 successfully pushed AUR commit 5807915 for 0.17.0. Parent verified remote PKGBUILD/.SRCINFO at 0.17.0-1 and matching archive checksum. 0.16.3/0.16.2 AUR workflows also show success. The temporary skip/read-only guidance is now misleading recurring release friction. Capture only; do not expand the current completed release to fix it.
