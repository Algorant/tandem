---
id: task-49-6
type: task
title: "Step 6: automatic sync in CLI, TUI, and web; remove checkpoint"
priority: "high"
effort: "medium"
parentId: "task-49"
tags: ["protocol", "sync", "tui"]
accord:
  status: "accepted"
  acceptance: ["Mutations publish immediately with a bounded timeout and report saved-locally-pending when offline; reads refresh when older than the freshness window and warn when stale; TUI and web sync on start, after mutations, and periodically.", "tandem checkpoint, --consolidate, and the batched checkpoint output are removed; lifecycle outputs report sync status instead."]
  claimedAt: "2026-09-30T04:51:40Z"
  deliveredAt: "2026-09-30T04:51:40Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml", "Rendered TUI sync status verified in a Herdr pane."]
  summary: "CLI mutations publish immediately and report data.sync with final IDs; reads refresh when older than 60 s and warn when stale or historical; TUI and web sync in the background (publish every 2 s, refresh every 30 s, 30 s back-off after failures); checkpoint and consolidation removed."
  evidence: ["`cargo test --manifest-path tandem/Cargo.toml`: 372 passed, 0 failed (318 unit + 54 integration, including 11 two-clone sync scenarios in tandem/tests/sync_behavior.rs).", "Herdr pane w38:pQ running the release TUI in clone A: a task added from clone B appeared within the 30 s refresh (Board (2)); a hand edit made in A while the TUI ran was published and visible from B within 5 s; A's source git status stayed clean.", "`tandem checkpoint` no longer exists (usage error); cli_behavior covers the new help surfaces (31)."]
  filesChanged: ["tandem/src/cli/commands.rs", "tandem/src/cli/model.rs", "tandem/src/cli/landing.rs", "tandem/src/app/background_sync.rs", "tandem/src/tui/mod.rs", "tandem/src/web.rs", "tandem/src/main.rs", "tandem/src/app/accord.rs", "tandem/src/app/review.rs", "tandem/src/app/project.rs"]
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

