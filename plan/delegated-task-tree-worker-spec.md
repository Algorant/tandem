# Delegated task-tree worker specification

Status: accepted working direction; canonical Tandem roles implemented, Pi-config handoff remains separate
Date: 2026-07-15
Related: `decision-7`, `task-134`, `extensions/pi-tandem/`

## Purpose

A worker delegated one Tandem **Task** should execute that Task's existing **Subtasks** as a structured campaign without requiring the parent orchestrator to wake up and delegate every Subtask separately.

Subtasks remain full Tandem task documents with parent-derived IDs, descriptions, dependencies, and history. They also serve as the delegated Task worker's concrete checklist. The worker projects those Subtasks into `pi-todos`, advances through them like the steps of any other multi-step implementation, and reports one Task-root handoff when the campaign is ready for parent review.

This is not a change from first-class Subtask documents to inline checklist strings. It is an execution and presentation model over decision-7's canonical hierarchy and nomenclature.

## Initial scope

Choose the simplest useful implementation:

- Only Task-role documents with global `task-N` IDs are delegation roots.
- Epics are not delegated; their globally numbered Tasks are delegated independently.
- Subtasks are not independently delegated; Worker A directly executes the delegated Task's Subtasks.
- Worker A does not create Worker B or use nested Shep/Herdr delegation in the initial version.
- Nested delegation, descendant-scoped Shep permissions, and nested settlement routing are deferred.
- The parent remains interactive while Worker A runs asynchronously.
- Worker A produces one settlement for the delegated Task after completing its live todo projection.

Under decision-7, the hierarchy and ID boundary are:

```text
task-134       Epic (not delegatable)
├── task-139   Task (delegatable)
│   └── task-139-1   Subtask (Worker A checklist item)
└── task-140   Task (delegatable)
```

Delegating `task-139` gives Worker A its direct Subtasks. A Task without Subtasks remains an ordinary one-item delegation.

## Sources of truth

### Tandem

Tandem is the durable source of truth for:

- task identity and hierarchy;
- descriptions and acceptance criteria;
- blockers and dependency order;
- related files and references;
- root accord, delivery, validation, acceptance, and completion;
- final Subtask reconciliation after Task-root review.

### pi-todos

`pi-todos` is the worker-session execution projection for:

- which Subtask checkpoint is currently active;
- which Subtasks are not started or completed in the worker's campaign;
- concise live progress in the worker TUI and parent Shep widget.

Todo titles should retain Tandem Subtask identity, for example:

```text
task-139-1 — Validate hierarchy role classification
```

Todo descriptions should include the child task's acceptance criteria, blockers, relevant files, and expected validation. Exactly one item should normally be `in-progress`.

The todo projection is not a second durable project database. It may be reconstructed from the Tandem tree when a worker session starts or resumes.

## Delegation behavior

When delegating a Task with Subtasks, Shep should:

1. Confirm the root is a valid Task role with a global `task-N` ID; reject Epic, Subtask, missing-parent, cyclic, and role/ID-mismatched roots.
2. Resolve the complete direct worklist from the Task's `show --json` `data.subtasks`, which aggregates Board and Logs and identifies each summary with `location` plus `completedAt` when completed.
3. Retain an exact-parent active read for the executable subset, then fetch active detail with `show` and completed context with `log show`; include each active Subtask's ID, title, state, body/description, blockers, references, related files, and relevant accord/review context in the worker prompt.
4. Present a dependency-valid recommended execution order.
5. Tell Worker A to create a matching `pi-todos` list before implementation.
6. Tell Worker A to update the whole todo list as checkpoints start and complete.
7. Request a focused commit per independently reviewable Subtask when practical, while allowing one commit for tightly coupled changes.
8. Require final evidence grouped by Subtask plus aggregate Task validation.

The delegated Task is the external review boundary. Worker A may implement all of its Subtasks directly in the prepared checkout, but it must not self-accept or complete the delegated Task.

## Initial lifecycle model

The simplest initial implementation does not grant tracked workers Tandem mutation/lifecycle tools.

- Worker A records live checkpoint progress in `pi-todos`.
- Descendant Tandem documents remain durable planning records during campaign execution.
- Worker A's final settlement reports each Subtask as completed, blocked, skipped, or requiring follow-up, with commit and validation evidence.
- The parent inspects and integrates the Task campaign once.
- After successful Task integration and acceptance, the parent reconciles Subtask records in dependency order and completes/logs them as a batch when appropriate.
- If the Task requires rework, no Subtask is prematurely archived as shipped.

A future version may permit descendant-scoped `tandem_task` and `tandem_accord` mutations, but only with programmatic enforcement that prevents a worker from accepting/completing its delegated root or mutating ancestors, unrelated siblings, and unrelated tasks.

## Shep and todo visibility

The existing Pi extensions already provide much of the desired presentation:

- `pi-todos` persists a worker's live multi-step list in session history.
- `pi-herdr` extracts the latest in-progress Todo List item from worker output.
- The Shep widget can therefore show the current Tandem child checkpoint while Worker A runs.

The initial Pi-config change should enable `manage_todo_list` for tracked workers. It should not enable nested `shep_*`, raw `herd_*`, or Tandem lifecycle mutation tools in the first version.

Expected widget activity resembles:

```text
● task-139  task-139-1 — Validate hierarchy role classification
```

The exact compact rendering remains a Pi-config concern; Tandem only needs stable IDs and child metadata.

## Checkout and commit behavior

- Worker A uses the checkout prepared for the delegated root.
- All Subtask implementation commits remain on that campaign branch/worktree until parent integration.
- Tandem coordination state may live in the source workspace while code lives in the prepared checkout; prompts and tools must make those paths explicit.
- The worker reports branch/worktree, clean status, commits grouped by Subtask, unexpected files, exact validation, and remaining risks.
- No Subtask should be logged as shipped merely because its commit exists on an unmerged campaign branch.

## Root settlement

The root settlement should include:

- delegated Task ID;
- ordered Subtask result table;
- each Subtask's todo outcome;
- commit hash or no-commit reason per Subtask/group;
- changed files per Subtask/group;
- validation commands and results;
- blocked/skipped/follow-up Subtasks;
- aggregate git status and checkout information;
- human/visual validation still required;
- `READY FOR PARENT DELIVERY` or `NOT READY: <reason>`.

Receiving the settlement does not automatically merge, accept, complete, push, or clean up. The parent performs those operations after one aggregate review, while retaining the option to inspect any individual checkpoint commit.

## Canonical Pi-config consumption sequence

Tandem now emits decision-7's strict roles and relationships. The separate Pi-config implementation should consume that CLI output rather than reproduce final role classification or preserve invalid historical shapes. Incorrect records such as hierarchical direct Epic children are not compatibility inputs.

For a requested delegation root, Pi/Shep should:

1. Consume the resolved Tandem role/relationship and allow only a global-ID Task; reject Epic, Subtask, missing-parent, cyclic, and role/ID-mismatched roots.
2. Read the Task with `tandem show <id> --json` and use its computed `data.subtasks` as the complete direct-child index across Board and Logs. Preserve each summary's `location` and optional `completedAt` so completed context is not lost.
3. Query active exact children with `tandem list --parent <id> --json` (or the equivalent thin pi-tandem call), require every result to be a CLI-classified, parent-derived Subtask, and reconcile those IDs with the Board entries from `data.subtasks`.
4. Fetch full detail for each indexed child according to location: `tandem show <subtask-id> --json` for Board entries and `tandem log show <subtask-id> --json` for Logs entries. Include completed detail as campaign context, but project only active Subtasks into executable todos.
5. Derive a dependency-valid execution order from active blockers and reject deeper children because Subtasks are leaf-only.

Task `show` supplies the complete Board+Logs worklist; exact-parent `list` supplies the active execution set. Neither read authorizes Pi extensions to duplicate allocation, graph validation, or final relationship classification. After Pi-config changes are validated, reload/restart Pi and confirm a tracked fixture Task worker can call `manage_todo_list` and execute one Task-owned Subtask campaign.

## Tandem repository contract

Tandem provides and documents:

- decision-7's correct Epic → Task → Subtask classification;
- complete Task and Subtask summaries plus reliable exact-parent reads;
- blockers/references/related-file context in machine-readable output;
- stable `epic-task`, `subtask`, and generic `parent` relationship values that pi-tandem passes through without reclassification;
- tests demonstrating that one Task root can expose its dependency-ordered Subtask campaign;
- repository pi-tandem guidance describing Task-owned Subtask execution without duplicating protocol logic.

Additional Tandem implementation should remain data-oriented; worker execution policy belongs primarily in Pi/Shep. Exact-parent traversal remains the integration contract even if richer aggregate child output is added later.

## Pi-config implementation handoff

Apply this handoff in the canonical Pi configuration repository. Expected areas include:

- `extensions/pi-shep/index.ts`
  - stop disabling `manage_todo_list` for tracked workers;
  - allow only CLI-resolved, structurally valid Task roots;
  - fetch the complete Task-owned Subtask index from Task `show`, reconcile active entries through exact-parent list, and use per-ID `show`/`log show` detail reads according to location;
  - generate dependency-aware campaign instructions and an explicit Tandem-ID-prefixed todo projection;
  - reject Epic/Subtask roots and invalid role/ID combinations rather than adding compatibility behavior;
  - retain Task-root-only settlement and parent-owned final lifecycle;
- `extensions/pi-shep/README.md` and tests
  - replace the one-worker-per-Subtask assumption with delegated-Task campaign semantics;
  - test Task-root eligibility, Epic/Subtask rejection, direct Subtask prompt context, blockers/order, invalid structures, and tracked-worker todo access;
- `extensions/pi-todos/`
  - preserve the existing whole-list update model;
  - add no Tandem parsing or lifecycle behavior;
- `extensions/pi-herdr/herds.ts`
  - retain current Todo List activity extraction;
  - tighten rendering/tests only if Tandem-prefixed todo titles expose gaps;
- canonical delegation/verification prompts and AGENTS guidance
  - define a worker as owner of its delegated Task's Subtask worklist;
  - keep parent review at the delegated-Task boundary.

Do not modify personal dotfiles from the Tandem repository task. Use this document as the explicit cross-repository handoff, apply it in the Pi-config repository, and return to task-134 only after Pi has reloaded the validated extension changes.

## Deferred capabilities

The following are intentionally out of the initial implementation:

- Worker B or deeper nested delegation;
- tracked-worker `shep_*`/`herd_*` access;
- descendant-scoped Tandem mutation enforcement;
- automatic descendant accord acceptance/completion before root integration;
- background parent auto-merge or auto-accept;
- replacing Tandem child documents with inline checklist strings.

These can be reconsidered after direct task-tree execution demonstrates where additional autonomy is genuinely useful.

## Acceptance criteria for the combined behavior

- A parent delegates one valid Tandem Task and remains free to continue its own conversation.
- Epic and Subtask roots are rejected.
- Worker A receives enough direct Subtask detail to execute without repeated parent steering.
- Worker A creates and maintains a `pi-todos` projection of the Task's Subtasks.
- The Shep widget shows Worker A's current Tandem Subtask checkpoint.
- Worker A executes dependency-valid Subtasks directly and produces reviewable commits/evidence.
- Parent receives one Task-root settlement and performs one aggregate integration review.
- Subtasks are not prematurely archived before their campaign branch is accepted.
- No nested worker delegation or tracked-worker Tandem lifecycle mutation is required for the initial version; the Pi-config handoff is applied separately from Tandem repository work.
