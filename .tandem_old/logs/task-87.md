---
id: task-87
type: task
title: "Document ADR-compatible Decisions and update agent guidance"
priority: "high"
parentId: "task-81"
references: ["task-85", "task-86"]
relatedFiles: ["docs/concepts/index.md", "docs/protocol/index.md", "docs/cli/index.md", "docs/tui/index.md", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md"]
tags: ["docs", "decision", "adr", "extensions", "agents"]
createdAt: "2026-07-01T18:05:58Z"
updatedAt: "2026-07-01T20:15:25Z"
accord:
  status: "accepted"
  assignee: "shep-decision-docs"
  claimedAt: "2026-07-01T18:08:20Z"
  deliveredAt: "2026-07-01T18:29:37Z"
  deliverables: ["Branch: hp/task81-decision-docs in ../tandem-worktrees/hp-task81-decision-docs", "New docs/guides/decisions.md plus related docs/guidance updates", "Agent guidance emphasizes `tandem_decision` and decision-body ADR sections"]
  validation:
    commands: ["git diff --check: pass", "bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts: pass", "tandem_decision action=list --json: pass per worker evidence", "git diff stat: 20 tracked files, 206 insertions, 38 deletions; untracked new docs/guides/decisions.md present"]
  summary: "Documentation-only ADR-compatible Decision guide and agent guidance work accepted per user direction that documentation validation does not require manual review."
  evidence: ["shep_check task-87 showed worker done and no blockers", "Local verification run in /home/ivan/dev/projects/tandem-worktrees/hp-task81-decision-docs"]
  filesChanged: ["AGENTS.md", "README.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/extensions/index.md", "docs/guides/index.md", "docs/guides/decisions.md", "docs/protocol/index.md", "docs/reference/index.md", "docs/tui/index.md", "extensions/pi-tandem/README.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md"]
  reviewer: "Algorant/orchestrator"
  updatedAt: "2026-07-01T20:15:16Z"
completedAt: "2026-07-01T20:15:25Z"
completion:
  summary: "Completed documentation-only ADR-compatible Decision guide and agent guidance updates. User directed documentation validation tasks can be marked complete without manual review."
  validation: "Previously verified delivered docs/guidance changes and checks; documentation review waived by user."
  reviewer: "Algorant/orchestrator"
---

## Description

Document Tandem Decisions as ADR-compatible durable records without introducing a separate ADR type: recommended body template, metadata fields, status/supersession examples, CLI/TUI examples, and Pi guidance so agents use `tandem_decision` rather than inventing task lifecycle state for decisions.
