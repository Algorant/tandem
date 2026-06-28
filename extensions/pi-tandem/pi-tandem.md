# pi-tandem Agent Guidance

Use `pi-tandem` when a project has `.tandem/tandem.md` or when the user asks for durable project coordination in Tandem.

## Prefer the tools

- Use `tandem_status` to check `tandem` and workspace health.
- Use `tandem_task` for task list/show/add/move/complete.
- Use `tandem_accord` for ready/claim/deliver/accept/rework/block/fail transitions.
- Use `tandem_log` and `tandem_search` for completed-work history and project search.
- Use `tandem_rules` for project rules.
- Use `tandem_decision` for first-class decision documents.

Avoid editing `.tandem/board/*.md`, `.tandem/logs/*.md`, or `.tandem/tandem.md` directly unless the user asks for raw source repair or the CLI cannot perform the needed action.

## Relationship guidance

When decomposing or linking work, set Tandem relationship fields explicitly instead of burying relationships in prose:

- `parent` writes `parentId` and is for supertask/child hierarchy. Create or inspect the parent document first.
- `blockers` writes strict dependency IDs. Blockers must already exist; unresolved blockers are validation errors.
- `references` writes related Tandem document IDs such as decisions, sibling tasks, or completed logs. Prefer existing IDs even though unresolved references are only warnings.
- `relatedFiles` records project paths that help implementers/reviewers find relevant code or docs.
- `subtasks` creates lightweight checklist items inside one task. Use child tasks with `parent` when work needs its own owner, accord, review, or blockers.

Example tool pattern:

```text
tandem_task action=add title="Ship relationship UI" relatedFiles=["tandem/src/tui.rs"] subtasks=["Define display", "Review copy"]
tandem_decision action=add title="Relationship display policy" references=["task-1"]
tandem_task action=add title="Render blockers and references" parent="task-1" blockers=["task-2"] references=["decision-1"] relatedFiles=["tandem/src/tui.rs", "protocol/plan/spec.md"] subtasks=["Show parent", "Show blockers", "Show references"]
```

After creating linked work, inspect with `tandem_task show` and `tandem_search`. If relationship fields are present in the document but hard to see in CLI/TUI output, report that as a display UX gap; do not invent replacement fields.

## Lifecycle cautions

- Claim or deliver only when assigned/asked.
- Move delivered work to the `validation` workflow state for acceptance, rejection, requested changes, or human/product judgment. Treat existing `state: review` files as legacy reads, not the preferred new state.
- Do not accept accords, complete tasks, or archive work unless the user/orchestrator explicitly asks; automated validation evidence is not human/product acceptance.
- Treat logs as first-class completed-work history, not as a trash folder.
- Keep workflow state, accord status, and `review:` metadata distinct. Review metadata remains the place for reviewer decisions/status.

## Bootstrap behavior

If no workspace exists, ask before creating one. This MVP diagnoses missing workspaces but does not hide initialization policy inside the extension; use `tandem init --title <title>` only after user intent is clear.

## Adapter boundary

`pi-tandem` calls `tandem` through argument arrays. Tandem protocol behavior belongs in the Rust CLI/protocol docs, not in TypeScript adapter logic.
