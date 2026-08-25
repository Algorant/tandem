---
id: papercut-7
title: "review.status not-ready has no command surface"
status: "resolved"
createdAt: "2026-08-24T23:28:14Z"
updatedAt: "2026-08-25T01:22:01Z"
references: ["decision-11", "task-239-1", "task-239-2"]
tags: ["cli", "protocol", "review", "v0"]
resolution:
  note: "Resolved in task-239-2. `not-ready` is now legacy-readable and themeable only; no code path in `tandem/src/` produces it. Symmetric with the existing treatment of `accord: ready`. Verified by grep after the change: the only remaining occurrences are the accepted-status list in protocol/review.rs and theme lookups in tui/theme.rs."
  resolvedAt: "2026-08-25T01:22:01Z"
---
## Observed

Found by the task-239-1 Worker and correctly reported rather than resolved by inventing a command.

`protocol/plan/spec.md` specifies `not-ready` as a valid `review.status` value at line 374 and in the review model example at line 818. After decision-11 the review command family is `tandem review request|accept|changes|reject`, none of which produces `not-ready`.

## Why it is not urgent

`not-ready` predates decision-11 and appears to duplicate an existing meaning. Under decision-11, a missing `review.status` is the normal state for work that has not been submitted for judgment, so `not-ready` and absent say the same thing.

## Options

- Make `not-ready` legacy-readable only, matching the existing treatment of `accord: ready` which the spec already documents as readable but not commandable.
- Remove it from the status vocabulary, since absent already covers it.
- Give it a command, which seems unwarranted for a state indistinguishable from absent.

Preference is the first: legacy-readable only, symmetric with `accord: ready`.

## Scope

Out of scope for task-239-1, which implemented decision-11 rather than auditing pre-existing review vocabulary. Worth settling during task-239-2 when the Rust implementation has to decide what to accept and emit.
