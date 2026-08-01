---
id: task-142
type: task
title: "Align repository pi-tandem guidance and smokes with canonical roles"
priority: "high"
blockers: ["task-140"]
references: ["decision-7"]
relatedFiles: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/README.md", "extensions/pi-tandem/plan", "extensions/pi-tandem/tests", "plan/delegated-task-tree-worker-spec.md"]
tags: ["pi-tandem", "typescript", "bun", "guidance", "smoke", "ids"]
createdAt: "2026-07-15T19:45:15Z"
updatedAt: "2026-07-22T03:20:02Z"
parentId: "task-134"
accord:
  status: "accepted"
  assignee: "shep-task-142"
  claimedAt: "2026-07-22T02:54:34Z"
  deliveredAt: "2026-07-22T03:19:10Z"
  deliverables: ["Thin kind/parent forwarding without TypeScript allocation or relationship classification", "Canonical epic-task, subtask, and generic parent guidance across repository-local pi-tandem surfaces", "Task-only delegation and Task-owned Subtask campaign handoff specification", "Canonical relationship/runtime smokes including both role/ID mismatch directions, invalid depth/nested Epic, reparent rejection, generic parents, and completed history", "Generated prompt-guidance assertions and corrected current planning/todo text"]
  validation:
    commands: ["bun --check on index.ts and all three smoke files — passed", "relationship-smoke.ts — passed against task-140 debug binary", "smoke.ts — passed; isolated worktree correctly skipped repository read and passed temporary mutations", "pi-runtime-smoke.ts — passed with temporary state cleaned", "cargo test --manifest-path tandem/Cargo.toml — 148 passed during parent review", "git diff --check 58d937a..bdafd730 — passed", "Final worktree clean; only 10 intended repository files changed"]
  summary: "Accepted after three parent review rounds, independent audit resolution, final Bun/static/runtime/relationship smokes, clean fast-forward integration at bdafd73, and successful repository-read verification against the migrated real workspace."
  evidence: ["Commit bdafd730b5d95966ace6a03e1b391cd20b04b815", "Independent audit findings 2–4 resolved; stale public docs finding recorded as task-143-2", "No /home/ivan/.dotfiles or personal Pi configuration changes"]
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/plan/todo.md", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/tests/pi-runtime-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.md", "plan/delegated-task-tree-worker-spec.md"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-07-22T03:19:54Z"
completedAt: "2026-07-22T03:20:02Z"
completion:
  summary: "Integrated strict canonical pi-tandem hierarchy guidance, thin kind/parent forwarding, Task-only delegation handoff, completed-Subtask retrieval guidance, and comprehensive Bun smokes in bdafd73."
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/plan/todo.md", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/tests/pi-runtime-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.ts", "extensions/pi-tandem/tests/relationship-smoke.md", "plan/delegated-task-tree-worker-spec.md"]
  validation: "Three parent review rounds and independent audit resolved; Bun check plus relationship/repository/Pi runtime smokes passed post-merge; 148 Rust tests passed during review; git diff check passed; no personal Pi config or /usr/bin changes."
---

## Description

This is a direct Task of Epic task-134. Keep pi-tandem thin while correcting all repository-local integration language and verification.

Acceptance criteria:
- Update tool descriptions, injected Tandem guidance, pi-tandem.md, README, plan, and examples to say Epics contain global Tasks and Tasks contain parent-derived Subtasks.
- Continue passing `parent` directly to Tandem; do not duplicate allocation or final role classification in TypeScript.
- Consume CLI-returned `epic-task`, `subtask`, and generic `parent` relationships.
- Correct delegation guidance: only Tasks are delegated; a Task worker owns its Subtasks through the todo projection; Epics and Subtasks are not delegation roots initially.
- Update Bun relationship/runtime smokes for Epic → global Task → parent-derived Subtask output, generic parents, invalid nested Epics, invalid children beneath Subtasks, role-changing reparent rejection, and no compatibility for erroneous Epic hierarchical children.
- Reference the cross-repository Pi-config handoff spec without modifying personal dotfiles.
