---
id: task-49-1
type: task
title: "Step 1 spike: validate sync mechanics in throwaway repositories"
priority: "high"
effort: "medium"
parentId: "task-49"
tags: ["protocol", "sync", "spike"]
accord:
  status: "accepted"
  acceptance: ["Measured overhead of writing the safety copy (refs/tandem/pending) on a board the size of the largest real workspace, and confirmed it never appears in source git status, log, index, or branches.", "Demonstrated restore of unsynced changes after an old-commit checkout and after git clean -fdx.", "Demonstrated snapshot, fetch, compare-and-swap push with retry, fresh-clone hydration, and linked-worktree resolution to the main worktree board using Git plumbing only.", "Read-only inventory of copies of this repo, ~/.dotfiles, and ~/.pi: record counts, pending legacy .tandem state, existing collisions, and anything the migration must special-case."]
  claimedAt: "2026-09-30T03:55:22Z"
  deliveredAt: "2026-09-30T03:57:17Z"
  validation: ["Re-runnable spike scripts and their output are recorded with the findings."]
  summary: "Spike confirmed the sync mechanics in disposable repositories and inventoried the three real workspaces; no design change needed."
  evidence: ["Inventory (read-only): tandem 92 files/584KB, ~/.dotfiles 70/424KB, ~/.pi 403/2.7MB; all 0.3.0, single worktree, up to date, no duplicate IDs; only this repo has pending .tandem changes.", "Safety copy on a copy of the ~/.pi board: 50 ms cold, 20-25 ms incremental; source status, index, branches, and log unchanged.", "Old-commit checkout replaced a board file and returning deleted it; git clean -fdx deleted the board; restore from refs/tandem/pending took ~25 ms and reproduced the exact tree including an unsynced file.", "Plumbing publish, fresh-clone hydration, stale push rejection without force and success after fetch, and linked-worktree main-board resolution all verified.", "GitHub round trip 0.6-0.9 s: publication will push optimistically on refs/tandem/base and fetch only on rejection. GIT_INDEX_FILE must be absolute.", "Script: /tmp/tandem-sync-review-tYjihD/spike-step1.sh; output spike-step1.txt and spike-step1-cd.txt. Findings in protocol/plan/independent-metadata-sync.md section 17."]
  filesChanged: ["protocol/plan/independent-metadata-sync.md"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-09-30T03:57:17Z"
createdAt: "2026-09-30T03:54:17Z"
updatedAt: "2026-09-30T03:57:17Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-30T03:57:17Z"
resolution:
  outcome: "completed"
  reviewer: "pi-orchestrator"
---

## Description

Disposable state only; no live workspace changes. Findings are appended to protocol/plan/independent-metadata-sync.md and reviewed with Algorant before step 2.
