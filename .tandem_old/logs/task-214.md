---
id: task-214
type: task
title: "Strengthen `.tandem` commit grouping and squash guidance"
priority: "low"
references: ["task-210"]
relatedFiles: ["docs/guides/agents-and-adapters.md", "extensions/pi-tandem/pi-tandem.md"]
tags: ["docs", "guidance"]
createdAt: "2026-08-05T18:02:06Z"
updatedAt: "2026-08-05T18:05:11Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-08-05T18:04:01Z"
  deliveredAt: "2026-08-05T18:04:59Z"
  deliverables: ["docs/guides/agents-and-adapters.md", "extensions/pi-tandem/pi-tandem.md"]
  validation:
    commands: ["git diff --check passed", "Required lifecycle boundary language appears in both guidance files", "Required local/unshared squashing and shared-history safety language appears in both guidance files", "Adapter-facing guidance links to the framework-neutral contract"]
  summary: "Objective documentation criteria are satisfied. The wording explicitly covers grouping, local squashing, lifecycle checkpoints, and shared-history safety without changing framework authority boundaries."
  evidence: ["Reviewed focused two-file diff", "git diff --check passed", "All required phrases verified with ripgrep"]
  filesChanged: ["docs/guides/agents-and-adapters.md", "extensions/pi-tandem/pi-tandem.md"]
  reviewer: "pi"
  updatedAt: "2026-08-05T18:05:04Z"
assignee: "pi"
completedAt: "2026-08-05T18:05:11Z"
completion:
  summary: "Made `.tandem` commit grouping and squashing behavior explicit in framework-neutral and pi-tandem guidance, including lifecycle checkpoints and shared-history safeguards."
  filesChanged: ["docs/guides/agents-and-adapters.md", "extensions/pi-tandem/pi-tandem.md"]
  validation: "git diff --check passed; required lifecycle, squash, stash/discard, and history-safety language verified in both files."
  reviewer: "pi"
---

## Description

Follow up on completed task-210 by making the intended commit behavior explicit rather than relying on “grouping” to imply it.

## Required guidance

Use this behavior in the framework-neutral agent guidance:

> When `.tandem` is tracked, commit durable coordination changes at coherent lifecycle boundaries rather than after every Tandem command. Group coordination changes with related project work when they form one logical unit. Before integration, branch changes, session shutdown, or push, inspect pending and local-only commits. Squash related unshared commits when they represent one coordination unit and doing so preserves clear history. Never rewrite pushed or otherwise shared history without explicit authority. Never silently stash, discard, or partially commit Tandem state.

## Acceptance criteria
- Explicitly require prudent squashing of related local/unshared Tandem commits; do not rely on “group changes” to imply it.
- Name integration, branch changes, session shutdown, and push as natural inspection/checkpoint boundaries.
- Preserve the distinction between coherent lifecycle commits and one commit per Tandem command.
- Preserve shared-history safety and prohibit silent stash, discard, or partial Tandem commits.
- Keep the guidance framework-neutral.
- Update nearby adapter-facing guidance only where needed to point to or faithfully reflect the framework-neutral contract.
