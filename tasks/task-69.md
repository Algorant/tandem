---
id: task-69
uid: 10aea0ea-acfb-4e18-b718-0cc7bbdf0231
type: task
kind: "papercut"
title: "blockers accept only board IDs, so dependencies outside the board can only go in prose"
state: "in-progress"
priority: "low"
references: ["task-42", "decision-11"]
tags: ["protocol"]
accord:
  status: "rework"
  acceptance: ["A decision is recorded on whether `blockers` accept an external reference (another workspace's Task or free text) or external dependencies stay in prose plus `accord block`.", "The chosen answer is documented in protocol/README.md and the user docs."]
  claimedAt: "2026-10-12T16:18:19Z"
  deliveredAt: "2026-10-12T16:19:39Z"
  validation: ["just site-build", "cd site && bun run check:links"]
  summary: "Documented decision-11 in protocol/README.md (new paragraph in \"Accord and archive\") and docs/cli/index.md (new \"Dependencies outside the workspace\" subsection under `tandem accord`, with a short claim → block → resume example). No code, schema, or other files changed. Decision-11 already exists and is accepted, so the decision criterion is met."
  evidence: ["A decision is recorded on whether blockers accept external references or external dependencies stay in prose plus accord block: decision-11 is accepted: blockers stay workspace-local, external dependencies go in the Task body (optional absolute URL reference) plus explicit accord block/resume. Read via `tandem show decision-11`.", "The chosen answer is documented in protocol/README.md and the user docs: Both files now state it. Commit 7ac37f0. They cover blockers vs references vs blocked Accord vs state, claim-first being recommended but not mandatory, resume clearing the note, no auto-resume or fetch, and manual resolution.", "External blocker forms rejected (disposable board): `add task X --blocker other:task-1` and `--blocker https://example.com` both fail with `Validation failed: blocker document not found: ...`.", "Reference handling (disposable board): `--reference https://example.com/dep` and a path-like reference were accepted and stored in `references` without error.", "Block, resume and completion behavior (disposable board): `accord block` on an unclaimed Task succeeded: accordStatus blocked, state todo, assignee null, note kept. `resume` then gave claimed with note null. After claim → block, state stayed in-progress, assignee w, blockers [], accordStatus blocked. After resume the note was cleared. `complete` on a blocked Accord succeeded with `Warning: ... accord.status=blocked; complete normally follows a delivered Accord.`", "Assignment readiness (disposable board): A Task with no blockers and a blocked Accord reported root.ready true and dependencyReadiness {allClear:true, issues:[]}. A Task blocked by an active task-3 reported ready false with issue 'active blocker', and that did not change after claim → block. A blocker that was already archived to Logs (task-1) was accepted on `add`."]
  filesChanged: ["protocol/README.md", "docs/cli/index.md"]
  note: "Please make a small docs polish correction before acceptance: replace `--assignee ivan` with generic `--assignee agent-a` (never substitute Algorant's OS username). Make the public example tool-neutral: wait on an upstream SDK release rather than private ~/.pi/dbs tooling. Remove bare `See decision-11` / `(decision-11)` from public/normative text since the local board record isn't accessible to external readers; the decision remains recorded on the Board. Split the very long new protocol paragraph into a sh"
  updatedAt: "2026-10-12T16:19:41Z"
createdAt: "2026-10-12T12:35:08Z"
updatedAt: "2026-10-12T16:19:41Z"
effort: "small"
relatedFiles: ["protocol/README.md", "docs/cli/index.md"]
assignee: "worker-task-69-33474114"
---
## Approved implementation

Algorant via Herdra authorized deciding and documenting this Task. Decision-11 records the choice: retain workspace-local resolving blockers; external dependencies stay in the Task body with an optional absolute URL reference, plus explicit accord block/resume. No schema change, external-resolution service or automatic clearing.

Scope: small clarifications in protocol/README.md and docs/cli/index.md on main. The docs-overhaul branch/task-76 stays parked; this is an independently authorized policy clarification, not resuming the overhaul. Blocked Accord versus blockers and workflow state must be distinguished. Explain that references are loose context, assignment readiness is derived from blockers only, blocked Accord does not enforce completion eligibility, and resume clears the block note. Recommend claim → block → resume as a usage sequence, but do not invent a code requirement that claim is mandatory for block.

Original friction: p40 task-6 waited on external ~/.pi dbs tooling and codeparse pilot; workspace-local blockers cannot express those dependencies. Documentation should give a short realistic command example and explain manual resolution.