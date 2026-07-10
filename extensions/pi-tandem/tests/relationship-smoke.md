# pi-tandem Relationship Smoke Notes

Purpose: validate that pi-tandem exposes Tandem's first-class parent-linked task behavior without authoring deprecated inline checklist subtasks or reimplementing relationship logic in TypeScript.

## Controlled scenario

`relationship-smoke.ts` builds the current repository Tandem CLI, creates a temporary workspace, and uses pi-tandem argument builders plus `tandem` to create:

- a normal parent task with related files;
- a decision referenced by the parent/children;
- a fixture-prep child task linked through `parentId`;
- an implementation child with its own `parentId`, blocker, references, related files, and workflow state;
- a validation follow-up created as an ordinary task and then attached to the parent through `tandem update --parent`.

The smoke also verifies that the adapter schema omits deprecated inline `subtasks` authoring and that a legacy builder input is rejected instead of forwarding `--subtask`.

## Assertions

- Persisted children have `parentId`, blockers, references, and related files, with no newly-authored inline `subtasks` block.
- Child `show --json` returns `document.parentId` and CLI-computed `parentRelationship: "subtask"`.
- Parent `show --json` returns computed summaries for all parent-linked task children.
- `list --json` and `search --json` naturally retain the CLI's `parentId` and `parentRelationship` fields.
- Parent filters pass through for list/search and select the tracked children.
- Strict unresolved parents fail while unresolved loose references remain warnings.

## Adapter boundary

The adapter only builds argument arrays and returns Tandem output. Parent classification and computed subtask discovery remain in the Rust CLI/protocol implementation; pi-tandem does not parse Markdown, infer relationships from IDs, or synthesize response fields.
