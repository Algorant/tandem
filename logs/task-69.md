---
id: task-69
uid: 10aea0ea-acfb-4e18-b718-0cc7bbdf0231
type: task
kind: "papercut"
title: "blockers accept only board IDs, so dependencies outside the board can only go in prose"
priority: "low"
references: ["task-42", "decision-11"]
tags: ["protocol"]
accord:
  status: "accepted"
  acceptance: ["A decision is recorded on whether `blockers` accept an external reference (another workspace's Task or free text) or external dependencies stay in prose plus `accord block`.", "The chosen answer is documented in protocol/README.md and the user docs."]
  claimedAt: "2026-10-12T16:18:19Z"
  deliveredAt: "2026-10-12T16:20:15Z"
  validation: ["just site-build", "cd site && bun run check:links"]
  summary: "Docs polish applied in commit f7c93f1, on top of 7ac37f0. The example now uses `--assignee agent-a` and waits on an upstream SDK release, not private ~/.pi tooling. Every `decision-11` mention is gone from both files. The long protocol paragraph is split in two. No other scope changed."
  evidence: ["Generic assignee and tool-neutral example: The example in docs/cli/index.md uses `--assignee agent-a`, the body text 'Waits on the next release of an upstream SDK.', the reference https://example.com/sdk/issues/42, and the block note 'waiting on upstream SDK release'.", "No decision-11 reference in public or normative text: The `(decision-11)` heading text in protocol/README.md and the `See decision-11.` sentence in docs/cli/index.md are removed. The search found no `decision-11` in either file.", "Long protocol paragraph split: protocol/README.md now has a short external-dependency paragraph (workspace-local blockers; body prose plus optional URL reference; explicit block/resume; no fetching or auto-resume). A separate paragraph covers the lifecycle and readiness caveats: blocked Accord vs blockers and state, readiness driven only by blockers, references as loose context, completion still allowed with a warning, block not requiring claim but claim → block → resume recommended, resume clearing the note. The semantics are unchanged from the earlier version."]
  filesChanged: ["protocol/README.md", "docs/cli/index.md"]
  updatedAt: "2026-10-12T16:20:17Z"
createdAt: "2026-10-12T12:35:08Z"
updatedAt: "2026-10-12T16:20:17Z"
effort: "small"
relatedFiles: ["protocol/README.md", "docs/cli/index.md"]
assignee: "worker-task-69-33474114"
archivedAt: "2026-10-12T16:20:17Z"
resolution:
  outcome: "completed"
---
## Approved implementation

Algorant via Herdra authorized deciding and documenting this Task. Decision-11 records the choice: retain workspace-local resolving blockers; external dependencies stay in the Task body with an optional absolute URL reference, plus explicit accord block/resume. No schema change, external-resolution service or automatic clearing.

Scope: small clarifications in protocol/README.md and docs/cli/index.md on main. The docs-overhaul branch/task-76 stays parked; this is an independently authorized policy clarification, not resuming the overhaul. Blocked Accord versus blockers and workflow state must be distinguished. Explain that references are loose context, assignment readiness is derived from blockers only, blocked Accord does not enforce completion eligibility, and resume clears the block note. Recommend claim → block → resume as a usage sequence, but do not invent a code requirement that claim is mandatory for block.

Original friction: p40 task-6 waited on external ~/.pi dbs tooling and codeparse pilot; workspace-local blockers cannot express those dependencies. Documentation should give a short realistic command example and explain manual resolution.