---
id: task-127
type: task
title: "Align repository pi-tandem with hierarchical subtask allocation"
priority: "medium"
parentId: "task-101"
blockers: ["task-126"]
references: ["decision-4", "task-123"]
relatedFiles: ["extensions/pi-tandem"]
tags: ["pi-tandem", "subtasks", "ids", "smoke"]
createdAt: "2026-07-14T00:54:43Z"
updatedAt: "2026-07-14T02:41:41Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-14T02:31:54Z"
  deliveredAt: "2026-07-14T02:40:44Z"
  deliverables: ["Focused commit c5e33a45490e52aca2f202b0ccbaadaefec19355 on shep/task-127-align-repository-pi-tandem-with-hierarch", "Repository-local adapter guidance remains CLI-thin and hierarchical-ID aware", "Expanded relationship and Pi runtime smoke coverage plus docs/spec/todo updates"]
  validation:
    commands: ["Worker: Bun syntax checks passed", "Worker: repository smoke passed", "Worker: relationship smoke passed", "Worker: Pi runtime smoke passed", "Worker: 129 Rust tests passed", "Worker: git diff --check passed"]
  summary: "PASS. Parent reviewed the eight-file repository pi-tandem diff, confirmed the adapter remains CLI-thin, independently ran Bun syntax, standard smoke, relationship smoke, Pi runtime smoke, 129 Rust tests, and git checks, then fast-forwarded commit c5e33a45490e52aca2f202b0ccbaadaefec19355 to main."
  evidence: ["Clean worktree and exactly eight intended changed files", "Worktree /home/ivan/.pi/agent/worktrees/tandem/task-127-align-repository-pi-tandem-with-hierarch", "No unexpected files, risks, or blockers; ready for parent delivery"]
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/plan/todo.md", "extensions/pi-tandem/tests/relationship-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.md", "extensions/pi-tandem/tests/pi-runtime-smoke.ts"]
  reviewer: "pi"
  updatedAt: "2026-07-14T02:41:32Z"
completedAt: "2026-07-14T02:41:41Z"
completion:
  summary: "Aligned repository pi-tandem guidance and smoke coverage with CLI-owned hierarchical and nested subtask IDs, completed-log sequence continuity, generic parents, legacy flat children, collision errors, and deprecated inline-authoring boundaries."
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/plan/todo.md", "extensions/pi-tandem/tests/relationship-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.md", "extensions/pi-tandem/tests/pi-runtime-smoke.ts"]
  validation: "PASS. Parent reviewed and integrated c5e33a45490e52aca2f202b0ccbaadaefec19355; independently passed Bun syntax checks, standard smoke, relationship smoke, Pi runtime smoke, all 129 Rust tests, git diff --check, clean status, and thin-adapter scope review."
  reviewer: "pi"
---

## Description

Update the repository-local pi-tandem adapter, guidance, and smoke tests after the CLI implements decision-4.

Acceptance criteria:
- Keep pi-tandem thin: pass `parent` to Tandem and rely on CLI allocation/classification rather than constructing IDs in TypeScript.
- Verify child creation returns hierarchical IDs such as `task-103-1` and nested IDs where applicable.
- Preserve generic non-task parent behavior and existing flat-ID child compatibility.
- Update relationship and Pi runtime smokes for hierarchical allocation, nested child creation, completed-log sequence continuity, and collision errors exposed by the CLI.
- Keep inline checklist `subtasks:` legacy/read-only and do not forward deprecated authoring.
- Update repository-local adapter docs/spec text consistently.
