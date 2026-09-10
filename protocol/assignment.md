# Current assignment read

This document defines the native current-assignment projection for protocol
0.3.0 clients. It is a read projection, not a second task store and not a
workflow state.

## Why this is a separate read

The version-matched 0.12.3 native reads were first probed with disposable
records:

```sh
tandem --version
tandem show task-1 --json
tandem show task-1-1 --json
```

`show task-1` returns the complete root body and Accord definition, but its
`children` entries are summaries. `show task-1-1` returns one complete child,
so a consumer would need one process call per milestone and still would not
have one coherent root-plus-milestones snapshot. The existing whole-workspace
revision also includes progress, timestamps, and unrelated documents. That
proves the narrow gap: native reads had no complete, single-snapshot assignment
projection or definition-only freshness token. This command adds only that
projection; it does not duplicate the existing single-document `show` read.

## Command

```text
tandem assignment <task-id> --json
```

The command accepts a normal Task only. An Epic, Subtask, Decision, or missing
ID is rejected. The command reads one coherent Board-and-Logs hierarchy and
returns a JSON envelope:

```json
{
  "ok": true,
  "data": {
    "definitionToken": "ad1-...",
    "root": { "...": "complete Task definition" },
    "milestones": [{ "...": "complete Subtask definition" }],
    "dependencyReadiness": {
      "allClear": true,
      "issues": []
    }
  },
  "warnings": []
}
```

`root` and every `milestones` item contain the complete `body`, `title`,
`acceptance`, `constraints`, `plannedValidation`, `ownedScope` (the native
`relatedFiles` list), and `dependencies`. They also include derived
`attemptCount`, `reworkCount`, and `discardedCount` from the Task's Accord
events. `plannedValidation` is an array of `{ "kind": "command" | "manual", "text": "..." }` items: values beginning with `$ ` are commands with the prefix and following leading whitespace removed, and all other values are manual checks. They also include current
`location`, `state`, `accordStatus`, and archived `resolutionOutcome` where
available. No body or list is truncated. Milestones are the direct Subtasks of
the Task and are returned in native query order. Archived milestones remain
visible in the assignment with their archived outcome.

`definitionToken` is an opaque native token for the current definition. It
covers the root and direct milestones, including their IDs, hierarchy fields,
titles, bodies, owned scope, blockers, acceptance, constraints, and planned
validation. Members are canonically ordered by native ID before hashing, so
progress that changes presentation order cannot change the token. It excludes
workflow state, Accord status, assignee, evidence, progress notes, tags, loose
references, timestamps, resolution metadata, and unrelated documents.
Consumers compare the complete token for freshness and must not interpret its
bytes or manufacture a replacement token.

## Dependency derivation

Dependency information is derived at read time. Nothing is persisted. A node's
`ready` means only that its own native `blockers` are clear. The aggregate
`dependencyReadiness.allClear` means every included root/milestone node has
clear direct blockers. It is not a dispatch-eligibility, sequencing, or
workflow-readiness decision. For example, a root with no blocker can have
`root.ready: true` while a milestone blocked by another active milestone makes
`dependencyReadiness.allClear: false`. Consumers must not infer that all
milestones can start concurrently from either field.

Each dependency entry has an exact `id`, `status`, `ready`, and `reason`:

| status | ready | reason |
| --- | --- | --- |
| `missing` | false | `missing blocker` |
| `active` | false | `active blocker` |
| `completed` | true | `archived blocker: completed` |
| `canceled` | true | `archived blocker: canceled` |
| `failed` | true | `archived blocker: failed` |

All archived outcomes unblock, matching the existing native completion blocker
policy, which treats only Board blockers or missing IDs as unresolved.
`dependencyReadiness.issues` repeats every missing or active dependency with its
owning node ID. `references` are loose related links and are not prerequisites;
each value is a document ID or an absolute `http(s)` URL. Neither form appears
in `dependencies` or dependency issues, and URL references are never fetched or
rewritten.

## Delivery evidence

The protocol requires `accord deliver` to include at least one evidence entry
containing non-whitespace text. Native validation rejects absent, empty, and
whitespace-only evidence before writing the Task or its event. Meaningful
observational evidence is accepted. This implements the existing documented
protocol requirement and does not change the Accord API or lifecycle shape.

A source change does not update an installed `tandem` binary. Consumers must
check `tandem --version` and use a binary containing the commit that implements
this command before relying on this projection.
