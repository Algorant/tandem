---
id: task-17
type: task
title: "CLI: review accepts a criterion that is not in the accord"
state: todo
priority: "low"
tags: ["protocol", "papercut"]
accord:
  status: "ready"
  acceptance: ["Review rejects a criterion absent from the Task's current accord.acceptance, preserving the existing exact-criterion protocol requirement.", "A matching acceptance criterion successfully enters validation with its exact text; invalid requests do not mutate the record, events, or Git checkpoint state.", "Regression coverage exercises exact matching, mismatched text, empty input and the valid request path."]
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml --check"]
  constraints: ["Validation meaning belongs in protocol, invoked by the shared app review operation before writes. Do not weaken the normative exact-criterion requirement or add CLI-only enforcement.", "Do not modify cli/commands.rs or TUI code; parallel tasks own those paths.", "Submit a plan before editing despite small effort; wait for approval; ask unclear questions through worker_ask."]
  updatedAt: "2026-09-11T14:21:34Z"
createdAt: "2026-09-05T22:07:42Z"
updatedAt: "2026-09-11T14:21:34Z"
effort: "small"
relatedFiles: ["tandem/src/protocol/accord.rs", "tandem/src/app/review.rs", "tandem/tests/review_behavior.rs"]
---

## Description

## Description

Preserved from published tandem-v0.12.3 task-14 during reconciliation with local delegation proposals task-13–15. Original creation: 2026-09-04T01:36:35Z. Source commit: 735c4cf738a5f4b78c2c28f043ae64cd5461f90a.

Observed while checking whether Pi could rely on the CLI to catch a mistyped criterion. protocol/README.md says review requires the exact unresolved criterion, but this is not enforced.

Reproduction on 0.12.2: add task alpha with acceptance 'criterion one'; claim task-1 as me; run `tandem review task-1 --criterion 'totally made up' --note n --json`. The command succeeds, enters validation, and stores a criterion absent from accord.acceptance.

Not a Pi dependency. Either enforce the match or drop the protocol claim so specification and behavior agree.
