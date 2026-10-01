---
id: task-49-4
uid: 666f8a04-1d90-42d3-a95a-95f6b343a5a7
type: task
title: "Step 4: sync engine core (safety copy, fetch, publish, restore, status)"
priority: "high"
effort: "large"
parentId: "task-49"
tags: ["protocol", "sync", "git"]
accord:
  status: "accepted"
  acceptance: ["Every mutation updates refs/tandem/pending; sync snapshots through a private index, fetches the tandem branch, pushes without force with bounded retry, and applies locally and advances refs/tandem/base only after remote acknowledgement.", "Missing or damaged board files are restored from the safety copy and never synced as deletions; a pre-0.4.0 board from an old checkout is shown with a historical warning and never synced.", "Linked worktrees resolve the main worktree board; a clone without a board hydrates from the remote; no remote means a local-only board reported as such.", "tandem sync and tandem sync status report pending, offline, synced, conflicted, and held states; source HEAD, index, and working-tree bytes are unchanged by every sync."]
  claimedAt: "2026-09-30T04:51:40Z"
  deliveredAt: "2026-09-30T04:51:40Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml"]
  summary: "Sync engine in tandem/src/project/sync.rs: safety copy (refs/tandem/pending), snapshot through a private index, fetch, optimistic push with bounded retry, apply only after acceptance, restore after git clean or old checkout, historical-board detection, linked-worktree board resolution, hydration of fresh clones, and sync status."
  evidence: ["`cargo test --manifest-path tandem/Cargo.toml`: 372 passed, 0 failed (318 unit + 54 integration, including 11 two-clone sync scenarios in tandem/tests/sync_behavior.rs).", "Integration: fresh_clone_downloads_the_board_and_source_stays_clean, unsynced_changes_survive_git_clean_and_old_checkouts, linked_worktrees_share_the_main_board, independent_creation_on_two_machines_never_collides (push race resolved without force).", "Real-board rehearsal (release build, local bare remotes): add+publish 240-390 ms, fetch+merge path 340-545 ms, fresh read 14-40 ms; source git status clean throughout."]
  filesChanged: ["tandem/src/project/sync.rs", "tandem/src/project/git.rs", "tandem/src/project/write.rs", "tandem/src/project/events.rs", "tandem/src/project/mod.rs"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-09-30T04:51:40Z"
createdAt: "2026-09-30T03:54:17Z"
updatedAt: "2026-09-30T04:51:40Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-30T04:51:40Z"
resolution:
  outcome: "completed"
  reviewer: "pi-orchestrator"
---

