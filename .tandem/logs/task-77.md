---
id: task-77
type: task
title: "Specify Git-safe per-actor event logs"
priority: "high"
relatedFiles: ["protocol/plan/spec.md", "tandem/plan/spec.md", "tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/logs.rs"]
tags: ["protocol", "events", "git", "sync", "local-first"]
createdAt: "2026-07-01T16:05:24Z"
updatedAt: "2026-07-01T18:30:13Z"
accord:
  status: "accepted"
  assignee: "shep-events"
  claimedAt: "2026-07-01T18:07:22Z"
  deliveredAt: "2026-07-01T18:28:38Z"
  deliverables: ["Branch: hp/task77-events-spec in ../tandem-worktrees/hp-task77-events", "Updated protocol/design docs and guidance for per-actor event logs", "No Rust implementation changes in this lane"]
  validation:
    commands: ["git diff --check: pass", "rg verification found per-actor, actorName, `<actor>:<seq>`, legacy-read, and semantic-conflict language in protocol/docs", "git diff stat: 9 markdown/docs files, 67 insertions, 36 deletions"]
  summary: "Accepted verified spec/docs update for Git-safe per-actor event logs. Evidence covers required protocol language for per-actor storage, legacy reads, actor/seq identity, actorName, append ownership, and semantic-conflict limitations."
  evidence: ["shep_check task-77 showed final worker summary and no blockers", "Local verification run in /home/ivan/dev/projects/tandem-worktrees/hp-task77-events"]
  filesChanged: ["AGENTS.md", "README.md", "docs/concepts/index.md", "docs/protocol/index.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md", "tandem/plan/spec.md"]
  updatedAt: "2026-07-01T18:30:06Z"
completedAt: "2026-07-01T18:30:13Z"
completion:
  summary: "Completed Git-safe per-actor event-log specification update after verification."
  validation: "Verified branch hp/task77-events-spec with git diff --check and targeted rg checks for per-actor storage, legacy reads, actor/seq identity, actorName, append ownership, and semantic-conflict language."
  reviewer: "orchestrator"
---

## Description

Scope and update the Tandem protocol/design for Git-safe event sharing by replacing the single shared append hotspot with per-actor event logs.

Decisions to capture:

- New event storage should use multiple per-actor append logs under `.tandem/events/<actor_id>.jsonl` rather than a single shared `.tandem/events.jsonl`.
- Existing `.tandem/events.jsonl` should remain readable as a legacy event source during transition, but new writes should not append to it by default.
- Each writer uses an auto-generated unique canonical `actor_id`; if no existing actor ID can be found, Tandem may generate a new one rather than blocking. The primary goal is event-log disentanglement, not perfect machine recognition.
- Event records require `actor` and `seq` in addition to the existing minimal envelope. Event identity is `<actor>:<seq>`.
- Each actor must only append to its own event file. Readers aggregate all per-actor event logs plus any legacy global event log.
- `actorName` is optional cosmetic metadata. It may be populated from user/project configuration when available, but is never used as canonical identity or file ownership.
- Semantic conflicts between merged actor logs should be preserved and surfaced/detected; resolution UX and resolution events are out of scope for the minimum change.

Acceptance criteria:

- Protocol/spec documents describe the per-actor event-log layout and legacy-read behavior.
- Spec clearly distinguishes canonical `actor_id` from optional display `actorName`.
- Spec defines required event identity fields and the per-actor append rule.
- Spec notes that this solves Git file-level append conflicts but not all semantic conflicts.
- Follow-up implementation work can be created from the spec without re-litigating these decisions.
