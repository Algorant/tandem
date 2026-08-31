---
id: task-128
type: task
title: "Align canonical Pi and Shep guidance with hierarchical subtask IDs"
priority: "medium"
parentId: "task-101"
blockers: ["task-127"]
references: ["decision-4", "task-124"]
relatedFiles: ["/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-shep", "/home/ivan/.dotfiles/pi/.pi/agent/AGENTS.md", "/home/ivan/.dotfiles/pi/.pi/agent/prompts/delegate.md"]
tags: ["pi-tandem", "subtasks", "delegation", "config"]
createdAt: "2026-07-14T00:54:54Z"
updatedAt: "2026-07-14T04:02:52Z"
accord:
  status: "failed"
  assignee: "shep"
  claimedAt: "2026-07-14T03:29:15Z"
  reason: "User rejected the canonical-dotfiles scope: this epic must remain within the Tandem repository and must not interact with task-33 or personal dotfiles. No task-128 commit will be integrated."
  updatedAt: "2026-07-14T04:02:41Z"
completedAt: "2026-07-14T04:02:52Z"
completion:
  summary: "Closed unimplemented after the user rejected its canonical-dotfiles scope. No task-128 commit was integrated; task-33 and the shared dotfiles checkout are outside this Tandem-repository epic."
  validation: "Human scope correction: work must remain in the Tandem repository and must not modify personal/canonical dotfiles. The orphaned worker commit was intentionally abandoned and no repository changes were accepted."
  reviewer: "Algorant"
---

## Description

Promote the validated repository adapter behavior and update canonical Pi/Shep delegation guidance after task-127.

Acceptance criteria:
- Canonical pi-tandem remains thin and does not generate hierarchical IDs itself.
- Shep-created child tasks pass `parent` and report the CLI-allocated hierarchical ID and `parentId` context in prompts/handoffs/delivery summaries.
- Guidance uses examples such as `task-103-1` and treats existing flat-ID children as compatible legacy/current data.
- Inline checklist subtasks remain legacy read-only context, not delegation units.
- Reconcile canonical files under `/home/ivan/.dotfiles`; do not edit symlinked runtime paths, per-machine settings, or install binaries.
- Run canonical extension tests/config checks, create a focused dotfiles commit, and report `/reload` requirements.
