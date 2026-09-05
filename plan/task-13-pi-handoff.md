# Task 13 native assignment handoff

## Consumer contract

The native current assignment is available from a source build containing the
Task 13 changes with:

```sh
tandem --version
tandem assignment task-1 --json
```

The command returns one stdout JSON envelope:

```json
{
  "ok": true,
  "data": {
    "definitionToken": "ad1-...",
    "root": {
      "id": "task-1",
      "type": "task",
      "role": "task",
      "title": "...",
      "body": "...",
      "location": "board",
      "state": "todo",
      "accordStatus": "ready",
      "acceptance": ["..."],
      "constraints": ["..."],
      "plannedValidation": ["..."],
      "ownedScope": ["..."],
      "dependencies": [],
      "ready": true
    },
    "milestones": [],
    "dependencyReadiness": {"allClear": true, "issues": []}
  },
  "warnings": []
}
```

`root` and every milestone contain complete, untruncated body and definition
lists. A milestone is a direct native Subtask. Archived milestones remain in
the result and expose `location: "logs"` and `resolutionOutcome`.

Pi should compare `data.definitionToken` as an opaque value. It covers current
root/direct-milestone definition scope: IDs and hierarchy fields, title, body,
`relatedFiles`, `blockers`, acceptance, constraints, and planned validation.
Members are canonically ordered by native ID, so progress cannot change the
token by changing presentation order. It excludes state, Accord
status/evidence, assignee, progress/timestamps, resolution metadata, tags,
loose references, and unrelated Tasks.

`root.dependencies` and each milestone's `dependencies` are native blockers,
not loose references. Each entry is `{id,status,ready,reason}`. Missing and
active blockers are not ready and use exact reasons `missing blocker` and
`active blocker`. Archived completed/canceled/failed blockers are ready and use
`archived blocker: completed`, `archived blocker: canceled`, or
`archived blocker: failed`. `data.dependencyReadiness.issues` repeats every
missing or active blocker with its owning node ID. `root.ready` means only the
root's direct blockers are clear; `dependencyReadiness.allClear` means every
included node's direct blockers are clear. Neither field means that all
milestones can start concurrently or that the assignment is workflow-ready.

Human assignment output sends warnings to stderr as `Warning: ...`; JSON keeps
warnings in the stdout envelope. A source change does not update an installed
binary; the installed CLI must report a version/build containing the Task 13
commit. No release, installation, or push is part of this handoff.

## Why a new assignment read is justified

The version-matched 0.12.3 native probe used disposable records and exact JSON:

```sh
tandem --version
tandem show task-1 --json
tandem show task-1-1 --json
```

`show task-1 --json` returns the complete root body/Accord definition but only
child summaries. `show task-1-1 --json` returns one complete child. Existing
reads therefore require one process call per milestone and cannot provide one
coherent root-plus-milestones snapshot. The existing whole-workspace revision
also includes progress, timestamps, and unrelated documents. The assignment
command is the smallest justified native extension: one complete coherent
projection plus a definition-only token, not a duplicate single-document
`show` API or a new task store.

## Acceptance evidence

- **Complete retrieval:** `assignment_json_is_complete_for_long_root_and_ten_milestones`
  creates a native root and ten native Subtasks. It compares the assignment
  root body with native `show`, asserts the exact generated full body, and
  compares every milestone body with its native child `show` result. It asserts
  exact acceptance, constraints, planned validation, and owned scope for every
  milestone, including the final child.
- **Scope freshness:**
  `assignment_token_changes_only_for_current_definition_scope` independently
  edits root and milestone body, acceptance, constraints, validation, owned
  scope, and blockers and asserts a new token after each edit. It then claims
  and delivers the root, changes an unrelated Task, and archives a milestone,
  asserting stability. `assignment_token_is_stable_when_progress_reorders_multiple_milestones`
  uses three milestones and exercises claim, block, resume, deliver, and archive
  on different members while native presentation order changes; the token stays
  stable.
- **Dependency/readiness:** `assignment_readiness_reports_exact_native_blocker_reasons`
  creates native active, completed, canceled, failed, and loose-reference
  records, then exercises a missing blocker input. It asserts all five exact
  dependency IDs/statuses/reasons, two exact dependency issues, and that the
  loose reference is absent from prerequisites. Protocol tests define the
  missing/active/all-archived-outcomes policy; the policy matches native
  completion's existing unresolved-blocker behavior. Dependency state is
  derived and no readiness field is written.
- **Delivery evidence:** protocol and app tests reject absent, empty, and
  whitespace-only evidence without changing the record or events, and accept
  meaningful observational evidence. The existing `AccordOptions` signature
  remains unchanged; TUI comma-input parsing remains Task-15 ownership.
- **Consumer output:** `protocol/assignment.md` records the exact command,
  JSON fields, version-matched audit, token exclusions, truthful dependency
  terminology, blocker semantics, evidence rule, and binary availability
  warning. `protocol/README.md` registers the 24-leaf CLI tree and documents
  non-whitespace delivery evidence.

## Availability

Corrected source candidate commit: **`c29488acaddf36462c737db5a3666d37293e54ea`** (`fix(tandem): harden assignment freshness and evidence`).

The installed `tandem 0.12.3` binary is not changed by this source work. Pi
must use a build containing the corrected source commit; do not assume the
published/installed 0.12.3 binary has `assignment` or the evidence validation
until verified.
