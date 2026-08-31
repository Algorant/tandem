---
id: task-16
type: task
title: "Test pi-tandem task relationship guidance"
priority: "medium"
blockers: ["task-14"]
references: ["task-13", "task-14"]
tags: ["pi-tandem", "relationships", "smoke"]
createdAt: "2026-06-28T00:11:43Z"
updatedAt: "2026-06-28T02:39:55Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-06-28T02:18:41Z"
  deliveredAt: "2026-06-28T02:39:54Z"
  deliverables: ["test:extensions/pi-tandem/tests/relationship-smoke.md:relationship guidance smoke notes", "docs:extensions/pi-tandem/README.md:relationship examples or guidance updates"]
  validation:
    commands: ["bun extensions/pi-tandem/tests/relationship-smoke.ts"]
  constraints: ["Run after task-14 project-local pi-tandem smoke is available.", "Do not invent new protocol relationship fields unless protocol docs are explicitly updated."]
  summary: "Added pi-tandem relationship guidance and deterministic relationship smoke coverage."
  evidence: ["Merged 243123a via 7d114a6; integrated validation passed."]
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/tests/relationship-smoke.ts"]
  reviewer: "pi"
  updatedAt: "2026-06-28T02:39:54Z"
completedAt: "2026-06-28T02:39:55Z"
completion:
  summary: "Integrated pi-tandem relationship guidance and smoke coverage; identified CLI/TUI relationship-display gap."
  validation: "bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts; bun extensions/pi-tandem/tests/smoke.ts; bun extensions/pi-tandem/tests/pi-runtime-smoke.ts; bun extensions/pi-tandem/tests/relationship-smoke.ts"
---

## Description

Test whether Pi/Tandem guidance causes agents to use Tandem relationship fields well once `pi-tandem` is project-locally testable.

Context:
- Tandem already has protocol fields for task relationships: `parentId`, `blockers`, `references`, related files, and subtasks.
- The current issue may not be missing protocol support, but weak agent guidance and weak Pi extension ergonomics.
- This should be tested after the `pi-tandem` project-local smoke path exists, so agents can use the actual Pi tools rather than ad hoc markdown edits.

Acceptance direction:
- Create a small test project or controlled workspace scenario with a parent/supertask, child tasks/subtasks, blockers/dependency chains, and related references.
- Use `pi-tandem` tools/guidance to ask an agent to plan and create linked work.
- Verify resulting Tandem documents use `parentId`, `blockers`, `references`, and subtasks correctly and visibly.
- Identify whether missing UX belongs in `pi-tandem` prompts/tool schemas, Tandem CLI/TUI display, or protocol docs.
- Update the pi-tandem docs/skill guidance with concrete relationship examples if needed.
