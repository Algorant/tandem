---
id: task-16
type: task
title: "Test pi-tandem task relationship guidance"
state: todo
priority: "medium"
blockers: ["task-14"]
references: ["task-13", "task-14"]
tags: ["pi-extension", "pi-tandem", "relationships"]
createdAt: "2026-06-28T00:11:43Z"
updatedAt: "2026-06-28T00:11:43Z"
accord:
  status: "ready"
  deliverables: ["test:extensions/pi-tandem/tests/relationship-smoke.md:relationship guidance smoke notes", "docs:extensions/pi-tandem/README.md:relationship examples or guidance updates"]
  validation:
    commands: ["rg -n 'parentId|blockers|references|subtasks|relationship' extensions/pi-tandem .tandem/board"]
  constraints: ["Run after task-14 project-local pi-tandem smoke is available.", "Do not invent new protocol relationship fields unless protocol docs are explicitly updated."]
  updatedAt: "2026-06-28T00:11:43Z"
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
