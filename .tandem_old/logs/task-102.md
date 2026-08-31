---
id: task-102
type: task
title: "Update docs for first-class subtasks"
priority: "medium"
parentId: "task-101"
relatedFiles: ["docs", "site/src/content/docs", "README.md"]
tags: ["docs", "subtasks", "relationships"]
createdAt: "2026-07-05T16:22:34Z"
updatedAt: "2026-07-14T00:55:26Z"
completedAt: "2026-07-14T00:55:26Z"
completion:
  summary: "Closed unimplemented because its optional/manual hierarchical-ID wording was superseded by accepted decision-4. Replacement documentation work is tracked in task-130 after protocol, CLI, integration, and Epic Board corrections."
  validation: "No task-102 implementation was performed. Human product clarification established automatic parent-derived IDs for new first-class subtasks; task-130 carries the corrected documentation scope."
  reviewer: "Algorant"
---

## Description

Document the subtask model for users and agents.

Scope:
- Explain that new subtasks are separate task documents using `parentId`.
- Explain when to use a parent task, an epic, or a child/subtask task.
- Mention hierarchical IDs as optional/recommended only when useful.
- Avoid documenting inline `subtasks:` as the preferred path going forward.
- Keep docs concise and non-jargony.
