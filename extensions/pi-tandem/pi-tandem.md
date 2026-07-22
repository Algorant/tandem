# pi-tandem Agent Guidance

Use `pi-tandem` when a project has `.tandem/tandem.md` or when the user asks for durable project coordination in Tandem.

## Prefer the tools

- Use `tandem_status` to check `tandem` and workspace health.
- Use `tandem_task` for task list/show/add/move/update/complete.
- Use `tandem_accord` for ready/claim/deliver/accept/rework/block/fail transitions.
- Use `tandem_log` and `tandem_search` for completed-work history and project search.
- Use `tandem_rules` for project rules.
- Use `tandem_decision` for first-class decision documents, including ADR-compatible durable records.

Avoid editing Tandem Markdown directly unless the user asks for raw repair or the CLI cannot perform a required action. `pi-tandem` passes argument arrays to Tandem; it does not parse frontmatter, allocate IDs, or classify relationships.

## Canonical hierarchy and relationships

Tandem derives roles from resolved documents and then validates their ID form:

```text
task-10       Epic: type=task, kind=epic; root global ID
└── task-11   Task: direct Epic child; global ID; epic-task
    └── task-11-1   Subtask: direct Task child; parent-derived ID; subtask
```

- An **Epic** is a root `type: task`, `kind: epic` document with a global `task-N` ID. It cannot have `parentId`.
- A direct child of an Epic is a **Task**, remains in the global `task-N` namespace, and has CLI-returned `parentRelationship: "epic-task"`.
- A direct child of a Task is a leaf **Subtask**, uses exactly `<Task ID>-M`, and has CLI-returned `parentRelationship: "subtask"`.
- A task parented by a decision/custom document remains a global-ID Task and has generic `parentRelationship: "parent"`.
- Subtasks cannot have children. Nested Epics, role/ID mismatches, and role-changing or ID-invalidating reparenting are structural errors.
- There is no compatibility exception for erroneous hierarchical IDs directly beneath Epics.

Set relationship fields explicitly:

- `parent` passes `--parent` directly to Tandem for add/update and filters exact `parentId` matches for list. Create or inspect the parent first, then use the CLI-returned ID and relationship. Never construct an ID or infer a role in Pi.
- `blockers` writes strict dependency IDs; unresolved blockers are errors.
- `references` writes loose related document IDs; unresolved references are warnings.
- `relatedFiles` records relevant project paths.
- Inline checklist `subtasks` metadata is legacy/deprecated and read-only. Use full Subtask documents beneath a Task for lifecycle-bearing checklist work.

Example:

```text
tandem_task action=add title="Ship relationship UI" kind="epic" relatedFiles=["tandem/src/tui.rs"]
# Suppose Tandem returns task-10.
tandem_task action=add title="Implement relationship display" parent="task-10" relatedFiles=["tandem/src/tui.rs"]
# Tandem returns a global Task such as task-11 with parentRelationship=epic-task.
tandem_task action=add title="Render relationship labels" parent="task-11" relatedFiles=["tandem/src/tui.rs"]
# Tandem returns a Subtask such as task-11-1 with parentRelationship=subtask.
```

Showing an Epic returns `data.tasks`; showing a Task returns `data.subtasks`; showing a Subtask returns no child collection. List/search/show relationship output comes from Tandem and must pass through unchanged.

## Delegation boundary

Only Task-role documents with global `task-N` IDs are delegation roots in the initial worker model.

- Do not delegate an Epic. Delegate its global Tasks independently.
- Do not delegate a Subtask. One worker delegated the parent Task owns its direct Subtasks as the worker-session `pi-todos` projection.
- The delegated Task is the single campaign settlement/review boundary. Subtasks remain durable Tandem documents, but are execution checkpoints rather than separate workers.
- Child/subagent workers report and deliver evidence only. They do not accept, complete, archive, push, merge, or clean up Tandem/Git state; the parent/orchestrator retains lifecycle authority.

See [`../../plan/delegated-task-tree-worker-spec.md`](../../plan/delegated-task-tree-worker-spec.md) for the repository contract and its explicit cross-repository Pi-config handoff. Keep Pi-config changes in that separate canonical repository; do not modify personal dotfiles from Tandem repository work.

## Epic convention

Create an Epic with `kind: "epic"` and no parent. Use `references` for loose decisions, sibling work, or logs; do not substitute references for hierarchy. Do not invent `type: epic`, `epic-N` IDs, ADR-style Epic records, custom folders, or special lifecycle states. Complete/archive an Epic through normal task completion only after its Tasks are done, canceled/superseded, or the owner decides the Epic is complete.

## Decision / ADR guidance

Use `tandem_decision` for durable project, product, and architecture choices. Tandem decisions remain `type: decision`; do not invent `type: adr`, decision task states, accord statuses, or completed logs for decisions.

Recommended body sections are Status, Context, Decision, Consequences, and Supersession. Keep ADR `status`, `date`, `deciders`, `supersedes`, and `supersededBy` as decision metadata/body content, not workflow `state`. Mirror supersession IDs in `references` when useful for search.

## Lifecycle cautions

- Claim or deliver only when assigned/asked.
- Move delivered work to `validation` for acceptance, rejection, requested changes, or human/product judgment. Treat existing `state: review` files as legacy reads.
- Keep workflow state, accord status, and `review:` metadata distinct.
- Do not accept accords, complete tasks, or archive work unless the user/orchestrator owns and requests that transition.
- Treat logs as first-class completed-work history.

## Bootstrap behavior

If no workspace exists, ask before creating one. Use `tandem init --title <title>` only after user intent is clear.
