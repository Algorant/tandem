# pi-tandem Agent Guidance

Use `pi-tandem` when a project has `.tandem/tandem.md` or when the user asks for durable project coordination in Tandem.

## Prefer the tools

- Use `tandem_status` to check `tandem` and workspace health.
- Use `tandem_task` for task list/show/add/move/complete.
- Use `tandem_accord` for ready/claim/deliver/accept/rework/block/fail transitions.
- Use `tandem_log` and `tandem_search` for completed-work history and project search.
- Use `tandem_rules` for project rules.
- Use `tandem_decision` for first-class decision documents, including ADR-compatible durable records.

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
tandem_decision action=add title="Relationship display policy" status="accepted" deciders=["Algorant"] references=["task-1"]
tandem_task action=add title="Render blockers and references" parent="task-1" blockers=["task-2"] references=["decision-1"] relatedFiles=["tandem/src/tui.rs", "protocol/plan/spec.md"] subtasks=["Show parent", "Show blockers", "Show references"]
```

After creating linked work, inspect with `tandem_task show` and `tandem_search`. If relationship fields are present in the document but hard to see in CLI/TUI output, report that as a display UX gap; do not invent replacement fields.

## Epic convention

Epics are ordinary tasks with a lightweight classifier:

```yaml
id: task-10
type: task
kind: epic
title: Ship documentation refresh
state: in-progress
```

When planning or decomposing an epic:

- Create/inspect the epic task first, then create child tasks with `parent` so the CLI writes `parentId`.
- Use `references` for loose related decisions, sibling tasks, or completed logs; do not use references as a substitute for hierarchy.
- Do not create `type: epic`, `epic-N` IDs, separate ADR/epic documents, custom folders, or special lifecycle states.
- Do not use `tandem_decision` for epics. Decisions/ADR-style records are only for durable choices.
- If the installed CLI cannot set `kind: epic`, do not invent a tool parameter. Use the normal task/parent relationship fields, then either report the metadata gap or make a minimal frontmatter edit only when explicitly needed and safe.
- Complete/archive an epic only through the normal task completion flow after its children are done, canceled/superseded, or the human/orchestrator decides the epic is complete.

Example child creation:

```text
tandem_task action=add title="Rewrite Concepts page" parent="task-10" references=["decision-3"] relatedFiles=["docs/concepts/index.md"]
```

## Decision / ADR guidance

Use `tandem_decision` for durable project, product, and architecture choices. Tandem decisions are ADR-compatible records using `type: decision`; do not invent `type: adr`, decision task states, accord statuses, or completed logs for decisions.

Recommended body shape:

```markdown
## Status

Accepted, proposed, superseded, deprecated, or rejected.

## Context

Why this choice is needed.

## Decision

What has been decided.

## Consequences

What changes because of it.

## Supersession

- Supersedes: decision-N or none
- Superseded by: decision-M or none
```

Example tool pattern:

```text
tandem_decision action=add title="Use Tandem decisions for ADRs" references=["task-87"] tags=["adr"] body="## Status\n\nAccepted.\n\n## Context\n..."
```

If ADR metadata such as `status`, `date`, `deciders`, `supersedes`, or `supersededBy` is needed, keep it as decision record metadata/body content, not workflow `state`. Mirror supersession IDs in `references` so current CLI/TUI search can find the relationship.

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
