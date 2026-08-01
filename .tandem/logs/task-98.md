---
id: task-98
type: task
title: "Trim Quickstart to a short first-use path"
priority: "medium"
parentId: "task-59"
references: ["decision-1"]
relatedFiles: ["site/src/content/docs/quickstart.md", "docs/quickstart.md"]
tags: ["docs", "quickstart", "content", "site"]
createdAt: "2026-07-04T23:25:48Z"
updatedAt: "2026-07-10T11:19:48Z"
accord:
  status: "delivered"
  assignee: "shep-quickstart"
  claimedAt: "2026-07-10T11:02:42Z"
  deliveredAt: "2026-07-10T11:08:34Z"
  deliverables: ["Agent-first Quickstart with explicit human and agent responsibilities.", "Prompt for the agent to inspect `tandem --help`, workspace rules, active tasks, and the CLI guide before editing.", "Safe test-task flow covering creation without starting, accord claim, implementation, validation, delivery, human TUI review, acceptance/rework, and archive.", "Sideshow preview: http://localhost:8228/session/nHL4VyJwOmA/s/rhciuW1F_h0"]
  validation:
    commands: ["Parent inspection: commit 7e2ca846082622a4155c8d2f3a2e5131205dbce9 changes only docs/quick-start/index.md and passes git show --check.", "Parent rerun: cd site && bun run check:docs — 15 pages built; 570 internal links passed.", "TUI hotkeys verified against tandem/src/tui.rs: A accept, R rework, C apply/archive accepted."]
  summary: "Reworked Quickstart into a 55-line agent-first flow that separates human TUI supervision from agent CLI execution and carries a safe test task through Todo, In progress, Validation, acceptance, and Logs."
  evidence: ["Focused commit 7e2ca846082622a4155c8d2f3a2e5131205dbce9 on main shared checkout.", "Worker reported clean git status and no unexpected files after commit.", "Sideshow surface rhciuW1F_h0, version 3; no user feedback received yet."]
  filesChanged: ["docs/quick-start/index.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-10T11:08:34Z"
completedAt: "2026-07-10T11:19:48Z"
completion:
  summary: "Discarded the attempted Quickstart rewrites after human review and restored docs/quick-start/index.md exactly to its pre-task state. No task-98 content was accepted or retained; the Quickstart direction will be rewritten separately by the user later."
  filesChanged: ["docs/quick-start/index.md"]
  validation: "Human explicitly rejected the task-98 direction. Commit 1def325 restores docs/quick-start/index.md to the exact content at pre-task commit 4191d90; `cd site && bun run check:docs` passed with 15 pages built and 593 internal links checked."
  reviewer: "user"
---

## Description

Keep the Quickstart useful, but cut it down hard.

Scope:
- Preserve the basic first-use flow if it still works.
- Remove verbose explanations, jargon, and conceptual detours.
- Prefer a few clear steps with commands, expected result, and next action.
- Do not turn Quickstart into a reference page or full product tour.
