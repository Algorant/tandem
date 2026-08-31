---
id: task-135
type: task
title: "Align event protocol and design with decision-6"
priority: "high"
references: ["decision-6", "decision-7", "task-77", "task-119"]
relatedFiles: ["protocol/plan/spec.md", "protocol/plan/todo.md", "protocol/README.md", "plan/spec.md", "tandem/plan/spec.md"]
tags: ["protocol", "events", "spec", "git"]
createdAt: "2026-07-15T19:42:43Z"
updatedAt: "2026-07-30T13:05:49Z"
parentId: "task-133"
completedAt: "2026-07-30T13:05:49Z"
completion:
  outcome: "canceled"
  summary: "Canceled: Superseded by decision-9 after the per-actor event ledger shipped in d974133. The planned union-ledger protocol rewrite is no longer the selected architecture."
---

## Description

This is a direct Task of Epic task-133. Update canonical protocol and implementation design documents to replace the unimplemented per-actor-file model with decision-6's single union-merged ledger.

Acceptance criteria:
- Specify `.tandem/events.jsonl` as the tracked append-only ledger and `.tandem/.gitattributes` rule `events.jsonl text eol=lf merge=union`.
- Define required new-record fields: UUIDv7 `eventId`, `ts`, `actor`, `event`, `id`, and `summary`; `actorName` remains optional display metadata.
- Remove `<actor>:<seq>` and per-worktree/per-actor file ownership as current requirements, while clearly documenting that task-77 was superseded before implementation.
- Define one random clone-local `tandem.actorId` shared across linked worktrees and ignored `.tandem/actor-id` fallback outside Git.
- Define mixed legacy/new reads, timestamp-plus-ID ordering, identical-ID deduplication, conflicting-payload corruption diagnostics, arbitrary union line order, append-only semantics, and no initial rotation.
- Cover semantic conflicts separately from text merge success.
- Update protocol todos/checklists consistently without claiming implementation has shipped.
