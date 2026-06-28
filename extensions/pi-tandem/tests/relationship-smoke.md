# pi-tandem Relationship Smoke Notes

Purpose: validate that pi-tandem guidance/tool schemas make Tandem relationship fields easy enough for agents to use without direct `.tandem` edits.

## Controlled scenario

`relationship-smoke.ts` creates a temporary Tandem workspace and uses pi-tandem argument builders plus `tandem` to create:

- a parent/supertask with `relatedFiles` and `subtasks`;
- a decision referenced by the parent/children;
- a fixture-prep child task with `parentId`, `references`, `relatedFiles`, and `subtasks`;
- an implementation child task with `parentId`, `blockers`, `references`, `relatedFiles`, and `subtasks`;
- a review follow-up task blocked by the implementation child.

It then verifies the generated Markdown contains `parentId`, `blockers`, `references`, `relatedFiles`, and parent-based subtask IDs, and verifies `tandem search` can find tasks by relationship metadata.

## Findings

- pi-tandem already passed relationship flags through to `tandem`, but the tool schema descriptions and prompt guidance were too terse for agent planning.
- Protocol docs already define the fields clearly; no new protocol fields are needed.
- CLI/TUI visibility remains the main UX caveat: `tandem show --json` currently returns identity/state/body details but not relationship fields. The smoke test therefore verifies raw persisted documents and search visibility, and docs now tell agents to report display gaps rather than invent replacement fields.

## Ownership of missing UX

- **pi-tandem prompts/tool schemas:** improved here with concrete field guidance and examples.
- **Tandem CLI/TUI display:** recommended follow-up: show relationship fields in `tandem show`/JSON and TUI task details.
- **Protocol docs:** no change needed for this smoke; field semantics are already documented.
