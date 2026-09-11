---
id: task-16
type: task
title: "CLI: init emits no JSON envelope on success"
state: "in-progress"
priority: "low"
tags: ["protocol", "papercut"]
accord:
  status: "claimed"
  acceptance: ["Successful tandem init --title X --json emits exactly one stdout JSON success envelope with ok, data, and warnings; stderr is empty on ordinary success.", "The non-JSON init path retains its current behavior; failure on an existing workspace retains the standard JSON error envelope and exit code.", "Regression tests exercise fresh success and repeated-init failure without altering real project records."]
  claimedAt: "2026-09-11T14:21:52Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml --check"]
  constraints: ["Only CLI init output and dedicated init regression tests; no adapter, protocol semantics, or unrelated command changes.", "Use the current native InitOutcome and established envelope conventions; no compatibility fallback for empty success.", "Submit a plan before editing despite small effort; wait for orchestrator approval."]
  updatedAt: "2026-09-11T14:21:52Z"
createdAt: "2026-09-05T22:07:28Z"
updatedAt: "2026-09-11T14:21:52Z"
effort: "small"
relatedFiles: ["tandem/src/cli/commands.rs", "tandem/tests/init_behavior.rs"]
assignee: "worker-task-16-f4a47543"
---

## Description

## Description

Preserved from published tandem-v0.12.3 task-13 during reconciliation with local delegation proposals task-13–15. Original creation: 2026-09-04T01:36:29Z. Source commit: 735c4cf738a5f4b78c2c28f043ae64cd5461f90a.

Observed while writing the Pi adapter parser against 0.12.2. `tandem init --title probe --json` creates a workspace, prints nothing, and exits 0. Repeating it produces a JSON error envelope for the existing workspace.

D53 says JSON is global with one shared envelope. An adapter should not need to special-case empty stdout plus exit 0 as success for this command.
