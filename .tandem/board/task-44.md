---
id: task-44
type: task
title: "Reuse workflow-state and accord sync in TUI mutations"
state: "validation"
priority: "high"
parentId: "task-26"
blockers: ["task-30"]
relatedFiles: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/plan/spec.md"]
tags: ["tui", "accord", "state", "validation"]
createdAt: "2026-06-28T17:29:46Z"
updatedAt: "2026-06-29T00:21:24Z"
accord:
  status: "delivered"
  assignee: "tasks44-45-accord-state"
  claimedAt: "2026-06-29T00:15:41Z"
  deliveredAt: "2026-06-29T00:21:24Z"
  deliverables: ["code:tandem/src/tui.rs:TUI mutation paths reuse shared workflow/accord synchronization", "code:tandem/src/main.rs:shared helpers stay canonical if additional factoring is needed", "tests:tandem:TUI/state synchronization coverage or smoke evidence"]
  validation:
    commands: ["cd tandem && cargo fmt --check", "cd tandem && cargo test", "cd tandem && cargo build"]
  constraints: ["Keep workflow state, review metadata, and accord status distinct.", "Avoid duplicating protocol mutation rules in TUI-only code.", "Coordinate with task-30 before touching Board Validation action paths."]
  summary: "Implemented shared move_task_to_state mutation helper so TUI Board moves reuse CLI task movement behavior, including ready -> claimed accord synchronization for todo -> in-progress moves. TUI move status now surfaces the accord sync result."
  evidence: ["cd tandem && cargo fmt --check && cargo test", "Commit 6ffa20a on branch herd-task44-45-accord-state"]
  filesChanged: ["tandem/src/main.rs", "tandem/src/tui.rs"]
  updatedAt: "2026-06-29T00:21:24Z"
---

## Description

Follow-up split from task-26.

Scope:
- Route TUI task movement and accord actions through the same conservative synchronization behavior used by the CLI.
- Keep workflow state, review metadata, and accord status distinct.
- Prefer claimed -> in-progress and delivered -> validation only when the current workflow state is compatible.
- Avoid duplicating protocol mutation rules in TUI-only code.

Traceability:
- Parent tracker: task-26.
- Replaces open subtask task-26-3.
- Coordinate with task-30 because Board Validation actions may touch the same TUI mutation paths.

Acceptance:
- TUI and CLI transitions no longer drift for common claim, deliver, rework, block, fail, accept, move, and complete flows.
- Add focused regression coverage or smoke evidence for TUI mutation behavior.
