# pi-tandem Relationship Smoke Notes

Purpose: validate that pi-tandem exposes Tandem's first-class parent-linked task behavior without authoring deprecated inline checklist subtasks or reimplementing relationship logic in TypeScript.

## Controlled scenario

`relationship-smoke.ts` builds the current repository Tandem CLI, creates a temporary workspace, and uses pi-tandem argument builders plus `tandem` to create:

- a normal parent task with related files;
- a decision referenced by the parent/children;
- a fixture-prep child and implementation sibling whose IDs are allocated as `<parent>-1` and `<parent>-2`;
- a nested implementation child allocated as `<parent>-2-1`;
- a validation follow-up created with a flat root ID and then attached through `tandem update --parent`, proving IDs remain immutable;
- a completed first child followed by `<parent>-3`, proving completed-log sequence continuity;
- a decision-parented task that retains flat allocation and generic `parentRelationship: "parent"`;
- a manually seeded legacy flat-ID child classified from `parentId`;
- occupied child destinations that force and expose the CLI's allocation collision error.

The smoke also verifies that the adapter schema omits deprecated inline `subtasks` authoring and that a legacy builder input is rejected instead of forwarding `--subtask`.

## Assertions

- The adapter forwards `--parent` and never constructs IDs; Tandem allocates hierarchical/nested children and flat generic-parent tasks.
- Persisted children have `parentId`, blockers, references, and related files, with no newly-authored inline `subtasks` block.
- Child `show --json` returns `document.parentId` and CLI-computed `parentRelationship: "subtask"`; generic and legacy relationships remain compatible.
- Parent `show --json` returns computed summaries for direct parent-linked task children, including nested summaries on child parents.
- `list --json` and `search --json` naturally retain the CLI's `parentId` and `parentRelationship` fields.
- Parent filters pass through for list/search and select the tracked children.
- Logged IDs are not reused; strict unresolved parents and exhausted destination reservations fail while unresolved loose references remain warnings.

## Adapter boundary

The adapter only builds argument arrays and returns Tandem output. ID allocation, collision handling, parent classification, and computed subtask discovery remain in the Rust CLI/protocol implementation; pi-tandem does not parse Markdown, infer relationships from IDs, construct IDs, or synthesize response fields.
