---
id: task-49-9
type: task
title: "Step 9: agent rollout and bootstrap guide"
priority: "high"
effort: "small"
parentId: "task-49"
tags: ["docs", "sync"]
accord:
  status: "accepted"
  acceptance: ["A guide covers: prerequisites (release installed on every machine; finish or discard Workers whose branches touched .tandem), rollout order (this repo, ~/.dotfiles, ~/.pi with its adapter update), first-machine migrate, other-machine adopt before pulling, new-machine bootstrap by cloning and running any tandem command, and checks to confirm success.", "It lists what agents must never do (hand-edit or delete .tandem to fix sync, pull before adopting, force-push the tandem branch) and how to handle held conflicts, historical-board warnings, and offline pending changes.", "It was followed successfully on disposable copies before release."]
  claimedAt: "2026-09-30T04:51:40Z"
  deliveredAt: "2026-09-30T04:51:41Z"
  summary: "docs/guides/upgrading-to-independent-sync.md gives the rollout and bootstrap steps for agents and people (prerequisites, order, first machine, other machines with and without unpushed work, new machine, day to day, never-do list); linked from the Workflows sidebar."
  evidence: ["Steps match the rehearsal and the migration integration test: migrate then push on the first machine; adopt before pull on machines with unpushed board changes; plain git pull otherwise; clone plus any command on new machines.", "`bun run check:docs` in site/: build succeeded; 993 internal links checked across 21 pages."]
  filesChanged: ["docs/guides/upgrading-to-independent-sync.md", "docs/guides/index.md", "site/astro.config.mjs"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-09-30T04:51:41Z"
createdAt: "2026-09-30T03:54:18Z"
updatedAt: "2026-09-30T04:51:41Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-30T04:51:41Z"
resolution:
  outcome: "completed"
  reviewer: "pi-orchestrator"
---

## Description

Plain guidance an agent (or Algorant) follows when the release drops. Not codified into tools; the ~/.pi adapter and sysup changes remain separate handoff Tasks in their own repositories.
