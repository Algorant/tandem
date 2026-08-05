---
id: task-210
type: task
title: "Add `.tandem` commit hygiene to agent guidance"
priority: "low"
relatedFiles: ["docs/guides/agents-and-adapters.md", ".tandem/tandem.md", "extensions/pi-tandem/pi-tandem.md"]
tags: ["docs", "guidance"]
createdAt: "2026-08-05T16:19:55Z"
updatedAt: "2026-08-05T16:23:25Z"
accord:
  status: "accepted"
  assignee: "worker-task-210-71a8e0e8"
  claimedAt: "2026-08-05T16:20:55Z"
  deliveredAt: "2026-08-05T16:23:17Z"
  deliverables: ["Added commit cadence, grouping, prudent local squashing, and shared-history safety guidance to docs/guides/agents-and-adapters.md.", "Updated plan/agent-adapter-implementation-handoffs.md to distinguish generic commit hygiene from repository-specific cadence."]
  validation:
    commands: ["git diff --check passed on Worker handoff and integrated commit.", "cd site && bun run check:docs passed: 18 pages built and 834 internal links checked."]
  summary: "Added framework-neutral `.tandem` commit hygiene guidance and reconciled nearby planning prose."
  evidence: ["Reviewed integrated commit 0118247 and accepted the wording against every task acceptance criterion.", "Adapter implementation files were not modified.", "Integrated via Worktrunk into main; target working tree is clean."]
  filesChanged: ["docs/guides/agents-and-adapters.md", "plan/agent-adapter-implementation-handoffs.md"]
  reviewer: "orchestrator"
  note: "Objective documentation work reviewed after integration. The wording satisfies all acceptance criteria, preserves the adapter boundary, and passes the full docs build and link check."
  updatedAt: "2026-08-05T16:23:22Z"
assignee: "worker-task-210-71a8e0e8"
completedAt: "2026-08-05T16:23:25Z"
completion:
  summary: "Added framework-neutral Git hygiene for durable `.tandem` data, including regular but coherent commits, prudent local squashing, and shared-history safety."
  filesChanged: ["docs/guides/agents-and-adapters.md", "plan/agent-adapter-implementation-handoffs.md"]
  validation: "Reviewed integrated diff; git diff --check passed; `cd site && bun run check:docs` passed with 18 pages built and 834 internal links checked."
  reviewer: "orchestrator"
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
