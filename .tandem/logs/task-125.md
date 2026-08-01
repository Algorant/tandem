---
id: task-125
type: task
title: "Correct protocol specification for hierarchical first-class subtask IDs"
priority: "high"
parentId: "task-101"
references: ["decision-4", "task-106"]
relatedFiles: ["AGENTS.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md"]
tags: ["protocol", "subtasks", "relationships", "ids"]
createdAt: "2026-07-14T00:54:17Z"
updatedAt: "2026-07-14T01:30:31Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-14T01:05:41Z"
  deliveredAt: "2026-07-14T01:29:46Z"
  deliverables: ["Focused amended commit 1241b670fd390ceb0f5ddecb8a59df4c4a2bf1e5 on shep/task-125-correct-protocol-specification-for-hiera", "Protocol and locked project documentation aligned with decision-4", "Requested rework resolved for reparenting, inline authoring, and arbitrary-depth filename semantics"]
  validation:
    commands: ["Worker: git diff --check HEAD^ passed", "Worker: targeted stale-wording searches found no contradictions", "Worker: positive checks confirmed --parent reparenting, rejected new inline authoring, arbitrary-depth IDs, and canonical parentId semantics", "Worker: exact five-file assertion and clean git status passed"]
  summary: "PASS. Parent reviewed the complete amended five-file protocol diff, confirmed all requested rework, independently checked stale and positive semantics, verified clean focused commit 1241b670fd390ceb0f5ddecb8a59df4c4a2bf1e5, and fast-forwarded it to main. Objective non-visual acceptance criteria are satisfied."
  evidence: ["Worktree /home/ivan/.pi/agent/worktrees/tandem/task-125-correct-protocol-specification-for-hiera", "Worker reported no unexpected files, risks, or blockers", "READY FOR PARENT DELIVERY"]
  filesChanged: ["AGENTS.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md"]
  reviewer: "pi"
  updatedAt: "2026-07-14T01:30:24Z"
completedAt: "2026-07-14T01:30:31Z"
completion:
  summary: "Corrected Tandem's protocol and locked project guidance so new first-class task children default to parent-derived hierarchical IDs while parentId remains canonical, existing flat children remain valid, IDs remain immutable, inline checklist authoring remains legacy, and nested allocation/reparenting semantics are consistent."
  filesChanged: ["AGENTS.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md"]
  validation: "PASS. Parent reviewed amended commit 1241b670fd390ceb0f5ddecb8a59df4c4a2bf1e5, verified the exact five-file diff and clean worker tree, ran git diff --check, checked merge compatibility, confirmed stale contradictory wording was absent and positive hierarchical/nested/reparent/inline-authoring semantics were present, then fast-forwarded the commit to main."
  reviewer: "pi"
---

## Description

Revise Tandem protocol and locked project language to implement decision-4 and correct the earlier task-101/task-106 interpretation.

Acceptance criteria:
- Define new first-class task children as normal task documents with `parentId` and default parent-derived IDs: `task-103-1`, `task-103-2`, with nested forms such as `task-103-1-1`.
- Keep `parentId` canonical; ID shape alone never establishes hierarchy.
- Existing flat-ID children remain valid without migration.
- Define child sequence allocation across active board and completed logs, without ID reuse.
- Define IDs as immutable and state that normal reparenting must not silently rename IDs/references.
- Distinguish first-class child tasks from legacy inline checklist `subtasks:`.
- Remove or supersede wording that says automatic hierarchical allocation is prohibited.
- Update relevant locked decisions/todos consistently without encoding implementation beyond the accepted decision.
