---
id: task-84
type: task
title: "Document Epic convention and update agent guidance"
priority: "high"
parentId: "task-80"
references: ["task-82", "task-83"]
relatedFiles: ["docs/concepts/index.md", "docs/protocol/index.md", "docs/cli/index.md", "docs/tui/index.md", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md"]
tags: ["docs", "epic", "extensions", "agents"]
createdAt: "2026-07-01T18:05:41Z"
updatedAt: "2026-07-01T20:15:09Z"
accord:
  status: "accepted"
  assignee: "shep-epic-docs"
  claimedAt: "2026-07-01T18:07:56Z"
  deliveredAt: "2026-07-01T18:29:10Z"
  deliverables: ["Branch: hp/task80-epic-docs in ../tandem-worktrees/hp-task80-epic-docs", "Updated protocol/docs/reference/concepts plus AGENTS.md and pi-tandem guidance", "No separate `type: epic`, epic ID allocator, command namespace, or lifecycle added"]
  validation:
    commands: ["git diff --check: pass", "bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts: pass", "git diff stat: 14 files, 144 insertions, 6 deletions"]
  summary: "Documentation-only Epic convention and agent guidance work accepted per user direction that documentation validation does not require manual review."
  evidence: ["shep_check task-84 showed worker done and no blockers", "Local verification run in /home/ivan/dev/projects/tandem-worktrees/hp-task80-epic-docs"]
  filesChanged: ["AGENTS.md", "README.md", "docs/concepts/index.md", "docs/guides/index.md", "docs/protocol/index.md", "docs/reference/index.md", "extensions/pi-tandem/README.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/plan/spec.md"]
  reviewer: "Algorant/orchestrator"
  updatedAt: "2026-07-01T20:15:01Z"
completedAt: "2026-07-01T20:15:09Z"
completion:
  summary: "Completed documentation-only Epic convention and agent guidance updates. User directed documentation validation tasks can be marked complete without manual review."
  validation: "Previously verified delivered docs/guidance changes and checks; documentation review waived by user."
  reviewer: "Algorant/orchestrator"
---

## Description

Document lightweight Epic support for humans and agents: examples using `type: task` plus `kind: epic`, parent/child task relationships through `parentId`, loose references, completion/archive expectations, and Pi guidance that avoids inventing a separate ADR/epic protocol behavior.
