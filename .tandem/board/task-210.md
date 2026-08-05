---
id: task-210
type: task
title: "Add `.tandem` commit hygiene to agent guidance"
state: "in-progress"
priority: "low"
relatedFiles: ["docs/guides/agents-and-adapters.md", ".tandem/tandem.md", "extensions/pi-tandem/pi-tandem.md"]
tags: ["docs", "guidance"]
createdAt: "2026-08-05T16:19:55Z"
updatedAt: "2026-08-05T16:20:55Z"
accord:
  status: "claimed"
  assignee: "worker-task-210-71a8e0e8"
  claimedAt: "2026-08-05T16:20:55Z"
  updatedAt: "2026-08-05T16:20:55Z"
assignee: "worker-task-210-71a8e0e8"
---

## Description

Add a focused section to the framework-neutral “Agents and adapters” guidance about committing Tandem workspace data.

Acceptance criteria:
- Explain that durable `.tandem/` workspace changes should be committed often enough to remain visible, portable, and safe from cleanup or reset.
- Explicitly avoid overkill: do not require one Git commit for every Tandem command or minor lifecycle mutation.
- Advise agents to group Tandem changes with the related project change or combine related coordination changes into a coherent commit when practical.
- Advise squashing related local/unpublished Tandem commits when prudent and possible, especially before handoff or integration.
- State that agents must not rewrite shared or published history without explicit authority.
- Keep the text framework-neutral and consistent with active workspace rule `prefer:7` and Tandem’s local-first model.
- Check nearby documentation for contradictory commit guidance and update links or wording only where needed.
