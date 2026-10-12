---
id: task-69
uid: 10aea0ea-acfb-4e18-b718-0cc7bbdf0231
type: task
kind: "papercut"
title: "blockers accept only board IDs, so dependencies outside the board can only go in prose"
state: todo
priority: "low"
references: ["task-42", "decision-11"]
tags: ["protocol"]
accord:
  status: "ready"
  acceptance: ["A decision is recorded on whether `blockers` accept an external reference (another workspace's Task or free text) or external dependencies stay in prose plus `accord block`.", "The chosen answer is documented in protocol/README.md and the user docs."]
  validation: ["just site-build", "cd site && bun run check:links"]
  updatedAt: "2026-10-12T16:18:04Z"
createdAt: "2026-10-12T12:35:08Z"
updatedAt: "2026-10-12T16:18:04Z"
effort: "small"
relatedFiles: ["protocol/README.md", "docs/cli/index.md"]
---
## Approved implementation

Algorant via Herdra authorized deciding and documenting this Task. Decision-11 records the choice: retain workspace-local resolving blockers; external dependencies stay in the Task body with an optional absolute URL reference, plus explicit accord block/resume. No schema change, external-resolution service or automatic clearing.

Scope: small clarifications in protocol/README.md and docs/cli/index.md on main. The docs-overhaul branch/task-76 stays parked; this is an independently authorized policy clarification, not resuming the overhaul. Blocked Accord versus blockers and workflow state must be distinguished. Explain that references are loose context, assignment readiness is derived from blockers only, blocked Accord does not enforce completion eligibility, and resume clears the block note. Recommend claim → block → resume as a usage sequence, but do not invent a code requirement that claim is mandatory for block.

Original friction: p40 task-6 waited on external ~/.pi dbs tooling and codeparse pilot; workspace-local blockers cannot express those dependencies. Documentation should give a short realistic command example and explain manual resolution.