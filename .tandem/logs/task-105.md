---
id: task-105
type: task
kind: "epic"
title: "Teach Pi/Shep delegation surfaces about subtasks"
priority: "medium"
references: ["task-101"]
relatedFiles: ["extensions/pi-tandem", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-shep/index.ts", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem/index.ts"]
tags: ["pi-tandem", "subtasks", "delegation", "validation"]
createdAt: "2026-07-05T16:22:34Z"
updatedAt: "2026-07-12T04:00:31Z"
accord:
  status: "accepted"
  assignee: "parent-orchestrator"
  claimedAt: "2026-07-10T19:32:32Z"
  deliveredAt: "2026-07-12T03:59:56Z"
  deliverables: ["task-123 repository adapter migration integrated as cf5c993", "task-124 canonical dotfiles/Shep integration committed as b9ec233c", "Updated automated adapter, relationship, Pi runtime, Shep linkage, config, and Stow validation coverage"]
  validation:
    commands: ["Both child tasks accepted and completed with recorded validation evidence", "Repository adapter Bun checks/smokes and 127 Rust tests passed", "Canonical extension builds/helper smoke, Shep link smoke, RPC config-check, Stow dry-run, and diff checks passed", "Tandem and dotfiles working trees are clean"]
  summary: "Accepted after parent review of both completed child tasks and their repository/canonical validation evidence."
  evidence: ["task-123 completed log", "task-124 completed log", "Tandem commit cf5c99397d38bb1cb842dcf2cb6699ad3bdb49de", "Dotfiles commit b9ec233cecd3e5f7f6ff289a6cb599c81253e5bd"]
  filesChanged: ["extensions/pi-tandem", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-shep", "/home/ivan/.dotfiles/pi/.pi/agent/AGENTS.md", "/home/ivan/.dotfiles/pi/.pi/agent/prompts/delegate.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-12T04:00:04Z"
completedAt: "2026-07-12T04:00:31Z"
completion:
  summary: "Completed Pi/Tandem and Shep adoption of first-class parent-linked subtasks across repository and canonical integrations, delegation prompts, relationship-aware handoffs/delivery summaries, guidance, and tests."
  filesChanged: ["extensions/pi-tandem", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-shep", "/home/ivan/.dotfiles/pi/.pi/agent/AGENTS.md", "/home/ivan/.dotfiles/pi/.pi/agent/prompts/delegate.md"]
  validation: "Accepted child tasks task-123 and task-124 provide full evidence: repository Bun smokes and 127 Rust tests passed; canonical extension builds/helper smoke, Shep linkage, RPC config-check, Stow dry-run, and diff checks passed; both repositories are clean."
  reviewer: "parent-orchestrator"
---

## Description

Adapt Pi/Tandem and Shep-facing guidance for the first-class subtask model.

Scope:
- Delegation prompts should understand subtasks as normal Tandem tasks linked by `parentId`.
- Delivery/validation summaries should report parent/subtask context where relevant.
- Avoid treating inline `subtasks:` checklist items as the future delegation unit.
- Keep pi-tandem thin over the CLI/protocol; do not duplicate protocol mutation logic in TypeScript.
- Update guidance/tests only where needed for delegation and validation surfaces.
