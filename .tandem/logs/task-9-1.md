---
id: task-9-1
type: task
title: "Persist accord.acceptance across accord transitions"
priority: "high"
effort: "small"
parentId: "task-9"
relatedFiles: ["tandem/src/protocol/accord.rs", "tandem/src/app/accord.rs", "tandem/src/tui/validation.rs"]
tags: ["protocol", "accord"]
accord:
  status: "accepted"
  acceptance: ["AccordRecord carries acceptance and every transition round-trips it", "claim, deliver, rework, block, resume, and release preserve acceptance criteria unchanged", "A regression test asserts acceptance survives a full transition cycle and fails without the fix", "TUI validation escalation reads the real criterion from accord.acceptance, covered by a unit test; the prompt's rendering defect is task-10"]
  assignee: "pi"
  claimedAt: "2026-09-01T04:51:29Z"
  deliveredAt: "2026-09-01T04:51:29Z"
  summary: "implemented and verified"
  evidence: ["cargo test: 244 passing", "rendered Herdr pane check"]
  updatedAt: "2026-09-01T04:51:29Z"
createdAt: "2026-09-01T04:33:50Z"
updatedAt: "2026-09-01T04:51:29Z"
archivedAt: "2026-09-01T04:51:29Z"
resolution:
  outcome: "completed"
---

## Description

## Reproduction

```
tandem add task alpha --acceptance "criterion one exactly"
# accord: { status: ready, acceptance: ["criterion one exactly"] }

tandem accord claim task-1 --assignee me
# accord: { status: "claimed", assignee: "me", claimedAt: ... }
```

The criterion is gone. No warning.

## Cause

`AccordRecord` (`tandem/src/protocol/accord.rs:6`) holds status, assignee, timestamps, deliverables, validations, constraints, summary, evidence, filesChanged, reviewer, note, reason. It has no `acceptance` field, so `AccordRecord::from_document` never reads it and every transition writes the accord block back without it.

## Impact

- D20 requires at least one acceptance criterion for a ready Task. The file stops satisfying that as soon as work starts.
- D17 requires the exact unresolved criterion to escalate to validation. It no longer exists.
- `tandem/src/tui/validation.rs:42` prefills the escalation criterion from `accord.acceptance` and falls back to the literal string "Confirm the agreed acceptance criteria". A human escalating a claimed task silently gets a placeholder where their criterion was.

## Direction

Change normative `protocol/` semantics first if the acceptance field is underspecified there, then add `acceptance` to `AccordRecord` and its document round-trip. Do not special-case individual transitions.
