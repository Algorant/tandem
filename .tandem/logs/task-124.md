---
id: task-124
type: task
title: "Update canonical Pi and Shep subtask delegation guidance"
priority: "medium"
parentId: "task-105"
blockers: ["task-123"]
references: ["task-103", "task-106"]
relatedFiles: ["/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem/index.ts", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-shep/index.ts", "/home/ivan/.dotfiles/pi/.pi/agent/AGENTS.md"]
tags: ["pi-tandem", "subtasks", "delegation", "config"]
createdAt: "2026-07-10T19:32:22Z"
updatedAt: "2026-07-12T03:58:55Z"
accord:
  status: "accepted"
  assignee: "shep-task-124"
  claimedAt: "2026-07-12T03:43:16Z"
  deliveredAt: "2026-07-12T03:58:38Z"
  deliverables: ["Focused canonical dotfiles commit b9ec233cecd3e5f7f6ff289a6cb599c81253e5bd on /home/ivan/.dotfiles master", "Canonical pi-tandem adapter/guidance aligned with Tandem commit cf5c993", "Shep parent-aware delegation creation, prompt context, check output, and delivery summary behavior", "Updated canonical AGENTS.md, /delegate prompt, and extension READMEs"]
  validation:
    commands: ["Parent inspected the complete six-file dotfiles commit diff", "Canonical pi-tandem and pi-shep Bun builds — passed independently", "Canonical parent/subtask builder, relationship context, delegation prompt, and delivery summary helper smoke — passed independently", "pi-shep smoke-parallel-links.sh task-124 from Tandem workspace — passed", "stow -n -v pi — simulation completed without conflicts", "RPC /config-check — worker evidence passed with no drift", "git diff HEAD^ HEAD --check — passed", "Dotfiles and Tandem working trees — clean; no unexpected files"]
  summary: "PASS. Parent reviewed the complete canonical dotfiles commit and independently validated extension builds, parent/subtask helpers, Shep linkage, Stow simulation, clean trees, and config-check evidence. Objective non-visual acceptance criteria are satisfied."
  evidence: ["Repository: /home/ivan/.dotfiles", "Branch: master (ahead of origin/master by one commit)", "Commit: b9ec233cecd3e5f7f6ff289a6cb599c81253e5bd", "Checkout mode: none; no separate branch/worktree", "Installed Tandem remains the existing 0.4.3 binary; newer relationship read/filter behavior requires a future validated binary update and was intentionally not installed", "Pi `/reload` or restart is required for extension/prompt changes"]
  filesChanged: ["pi/.pi/agent/AGENTS.md", "pi/.pi/agent/prompts/delegate.md", "pi/.pi/agent/extensions/pi-tandem/index.ts", "pi/.pi/agent/extensions/pi-tandem/README.md", "pi/.pi/agent/extensions/pi-shep/index.ts", "pi/.pi/agent/extensions/pi-shep/README.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-12T03:58:46Z"
completedAt: "2026-07-12T03:58:55Z"
completion:
  summary: "Aligned canonical Pi and Shep delegation with first-class parent-linked Tandem subtasks, including parent-aware task creation, CLI-provided relationship context in prompts/checks/deliveries, legacy inline-checklist guidance, and canonical pi-tandem parity."
  filesChanged: ["/home/ivan/.dotfiles/pi/.pi/agent/AGENTS.md", "/home/ivan/.dotfiles/pi/.pi/agent/prompts/delegate.md", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem/index.ts", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem/README.md", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-shep/index.ts", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-shep/README.md"]
  validation: "Parent reviewed canonical dotfiles commit b9ec233cecd3e5f7f6ff289a6cb599c81253e5bd. Canonical Bun builds and helper smoke passed; Shep link smoke passed; Stow dry-run, diff checks, RPC config-check evidence, and clean dotfiles/Tandem working trees were verified."
  reviewer: "parent-orchestrator"
---

## Description

Align the canonical shared Pi configuration and Shep/Tandem delegation surfaces with the accepted first-class subtask model after the repository-local adapter migration is validated.

Scope and acceptance criteria:
- Reconcile the canonical pi-tandem extension/guidance with the validated repository-local implementation rather than introducing independent protocol behavior.
- Update Pi/Shep prompts and handoff guidance so delegation units are normal Tandem tasks linked by `parentId`.
- Include parent/subtask context in delegation and delivery/validation summaries where the CLI exposes it.
- Stop presenting inline `subtasks:` checklist items as future delegation units while retaining any intentional legacy-read guidance.
- Keep edits in the canonical dotfiles repository, create a focused commit there, run extension tests/config checks, and report reload requirements.
- Do not install or rewrite per-machine settings as part of this task.
