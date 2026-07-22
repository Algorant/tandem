# pi-tandem Relationship Smoke Notes

Purpose: verify that pi-tandem passes canonical task metadata to Tandem and consumes Tandem's strict role/relationship output without implementing protocol logic in TypeScript.

## Controlled scenario

`relationship-smoke.ts` builds the current repository CLI, creates a temporary workspace, and uses pi-tandem argument builders to create:

```text
Epic global task-N
└── Task global task-M                 parentRelationship: epic-task
    ├── Subtask task-M-1               parentRelationship: subtask
    └── Subtask task-M-2               parentRelationship: subtask

decision-N
└── Task global task-P                 parentRelationship: parent
    └── Subtask task-P-1               parentRelationship: subtask
```

The first Subtask is completed and a later Subtask continues at `task-M-3`, proving per-Task suffix allocation scans logs.

## Assertions

- The tool schema exposes `kind`, `parent`, blockers, references, and related files, but not deprecated inline `subtasks` authoring.
- Generated prompt guidance states the canonical hierarchy, Task-only delegation boundary, and thin pass-through rule.
- Builders forward `--kind epic` and `--parent` exactly; they do not construct IDs or emit `--subtask`.
- A direct Epic child receives a global Task ID and CLI relationship `epic-task`, never an Epic-derived hierarchical ID.
- A direct Task child receives `<Task ID>-M` and CLI relationship `subtask`.
- A decision/custom-parented task remains a global Task with generic relationship `parent` and can own its own Subtasks.
- Epic show emits `tasks`; Task show emits Board+Logs `subtasks` with `location`/`completedAt`; Subtask show emits no child collection.
- List/search/show and exact-parent filters retain CLI-returned relationships unchanged.
- Persisted documents contain `parentId`, blockers, references, and related files without newly authored inline checklist metadata.
- Nested Epics and children beneath Subtasks fail.
- Reparenting that changes Task → Subtask role fails without mutation because IDs are immutable.
- A manually seeded hierarchical direct Epic child makes strict reads fail with `expected global task-N`; there is no compatibility exception.
- The inverse malformed shape—a global-ID child beneath a normal Task—also fails because a Subtask must use `<Task ID>-M`.
- Unresolved strict parents fail, while unresolved loose references remain warnings.

## Adapter boundary

The smoke uses manual malformed frontmatter only as an invalid-input fixture. Production pi-tandem code remains an argument-array adapter. Tandem owns document resolution, canonical role classification, allocation, graph validation, relationship values, and response collections.
