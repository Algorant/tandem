---
id: task-130
type: task
title: "Document hierarchical first-class subtasks"
priority: "medium"
parentId: "task-101"
blockers: ["task-125", "task-126", "task-127", "task-128", "task-129"]
references: ["decision-4", "task-102"]
relatedFiles: ["README.md", "docs", "site/src/content/docs"]
tags: ["docs", "subtasks", "relationships", "ids"]
createdAt: "2026-07-14T00:55:16Z"
updatedAt: "2026-07-14T04:56:07Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-14T04:15:22Z"
  deliveredAt: "2026-07-14T04:45:03Z"
  deliverables: ["Focused amended commit 785a06890df8cfe9cab59d3278faa1efbf33d99b on shep/task-130-document-hierarchical-first-class-subtas", "Supported tandem_task creation then shep_delegate existing-task flow", "Complete concepts, CLI, extension, and Epic Board public documentation"]
  validation:
    commands: ["Worker: Bun docs build passed, 15 pages", "Worker: Bun link check passed, 602 links", "Worker: git diff --check passed", "Worker: clean worktree and five-file scope"]
  summary: "PASS. Parent reviewed the amended five-file public documentation diff, confirmed supported Pi/Shep examples and finalized hierarchy/TUI terminology, independently built 15 docs pages and checked 602 links with Bun, and fast-forwarded commit 785a06890df8cfe9cab59d3278faa1efbf33d99b to main."
  evidence: ["Unsupported shep_delegate parent example removed", "Generic non-task parents distinguished from subtasks", "Approved SUB/state/parent-arrow/logged presentation documented", "READY FOR PARENT DELIVERY"]
  filesChanged: ["README.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/extensions/index.md", "docs/tui/index.md"]
  reviewer: "pi"
  updatedAt: "2026-07-14T04:55:58Z"
completedAt: "2026-07-14T04:56:07Z"
completion:
  summary: "Documented first-class hierarchical subtasks across public concepts, CLI, extension, TUI, and root README surfaces, including canonical parentId, nested allocation, immutable IDs, legacy flat compatibility, generic parents, supported delegation flow, and Epic Board presentation."
  filesChanged: ["README.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/extensions/index.md", "docs/tui/index.md"]
  validation: "PASS. Parent reviewed and integrated 785a06890df8cfe9cab59d3278faa1efbf33d99b; independently passed Bun docs build for 15 pages, 602 internal link checks, semantic documentation searches, git diff --check, clean status, and focused scope review."
  reviewer: "pi"
---

## Description

Replace task-102 after protocol, CLI, integration, and TUI corrections are accepted.

Acceptance criteria:
- Explain that a subtask is a full task document linked with `parentId` and normally designated with a parent-derived ID such as `task-103-1`.
- Explain nested IDs, sequence allocation, immutable IDs, reparenting behavior, and existing flat-ID compatibility.
- Explain when to use an epic, ordinary parent task, first-class child task, blocker, reference, or legacy inline checklist entry.
- Document CLI and Pi/Shep creation examples without implying adapters generate IDs themselves.
- Document Epic Board presentation and completed-child behavior after task-129 is approved.
- Keep public documentation concise and non-jargony; run Bun docs build/link checks.
