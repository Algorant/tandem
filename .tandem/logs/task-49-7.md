---
id: task-49-7
type: task
title: "Step 7: migration (tandem migrate and migrate --adopt)"
priority: "high"
effort: "medium"
parentId: "task-49"
tags: ["protocol", "sync"]
accord:
  status: "accepted"
  acceptance: ["tandem migrate --dry-run reports every change; tandem migrate preflights, adds uids and workspaceId, sets 0.4.0, publishes the tandem branch, hydrates the board, and creates one source commit that untracks and ignores .tandem while leaving the folder in place.", "tandem migrate --adopt brings a machine's uncommitted, untracked, and unpushed legacy changes onto the new board, renumbers colliding unpublished records with an old-to-new report, and holds ambiguous cases naming both sides.", "Existing IDs, references, Logs, Decisions, Rules, and event history remain valid; older Tandem refuses 0.4.0 boards."]
  claimedAt: "2026-09-30T04:51:40Z"
  deliveredAt: "2026-09-30T04:51:40Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml", "Dry-run and real migration of disposable copies of all three real workspaces."]
  summary: "tandem migrate [--dry-run] and tandem migrate --adopt implemented; older Tandem refuses 0.4.0 boards and 0.4.0 refuses unmigrated 0.3.0 boards with guidance."
  evidence: ["`cargo test --manifest-path tandem/Cargo.toml`: 372 passed, 0 failed (318 unit + 54 integration, including 11 two-clone sync scenarios in tandem/tests/sync_behavior.rs).", "Integration: migration_moves_a_legacy_board_and_adopts_unpushed_work_elsewhere (A migrates; B adopts an uncommitted edit and an untracked colliding task-2 -> renumbered task-3; B pulls; both converge; clean clone C downloads).", "Disposable rehearsal on clones of this repo (89 records), ~/.dotfiles (64), ~/.pi (384): migrate 230-510 ms, listed record counts equal to 0.14.1's, source clean after the migration commit, second machine downloads the board after git pull."]
  filesChanged: ["tandem/src/project/migrate.rs", "tandem/src/project/sync.rs"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-09-30T04:51:40Z"
createdAt: "2026-09-30T03:54:18Z"
updatedAt: "2026-09-30T04:51:40Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-30T04:51:40Z"
resolution:
  outcome: "completed"
  reviewer: "pi-orchestrator"
---

