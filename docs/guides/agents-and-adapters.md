---
title: Agents and adapters
description: Framework-neutral guidance for consuming Tandem safely.
---
Tandem gives agents and integration adapters one shared coordination contract. Repository `protocol/` documentation defines universal semantics. The active workspace rules in `.tandem/tandem.md` define repository policy. Each adapter owns its framework-specific tools, prompts, approvals, rendering, and diagnostics.

## Authority layers

Use these sources in order without merging their responsibilities:

| Authority | Owns | Does not own |
| --- | --- | --- |
| Tandem protocol | Documents, IDs, relationships, states, accords, reviews, Rules, Decisions, Logs, validation, and lifecycle operation semantics | Repository conventions or framework UX |
| Active workspace rules | Policy for the current repository | Universal Tandem behavior |
| Task and accord | Scope, assignee, constraints, deliverables, and recorded evidence for one unit of work | Permission beyond the assigned actor's authority |
| Adapter | Safe command mapping, structured output handling, prompts, rendering, and framework diagnostics | Protocol parsing, ID allocation, relationship classification, or lifecycle policy |
| Current caller or orchestrator | The actor's assignment and authority for the current operation | Changes to protocol meaning |

A more local source can add constraints, but it cannot redefine protocol fields or make an invalid operation valid. If instructions conflict, stop and surface the conflict instead of silently choosing an interpretation.

## Discover and read before mutation

1. Start from the intended project directory. Ask Tandem to discover the nearest workspace. The canonical marker is `.tandem/tandem.md`.
2. If no workspace exists, ask before initialization. Do not create coordination state only because an adapter is available.
3. Check protocol compatibility and workspace health. Do not perform an implicit upgrade or edit the version by hand.
4. Read all active rules before planning work.
5. Inspect the target document. Resolve its parent and blockers before creating strict relationships. Read references, related Decisions, related files, and Logs when they can change the plan.
6. Prefer Tandem's structured read output. Treat its IDs, roles, relationships, validation, and warnings as authoritative.

Do not infer a role from an ID. Resolve documents first. Tandem classifies Epics, Tasks, Subtasks, and generic parents, allocates canonical IDs, and validates the graph.

## Interpret rule categories

The stored category controls behavior. Do not classify a rule only from words such as “must,” “avoid,” or “usually.” Applicability and strength are separate.

| Category | How an actor applies it | Example |
| --- | --- | --- |
| `always` | Perform the required action whenever the rule applies. If compliance is impossible, report that fact. | “When changing the public API, update its compatibility note.” |
| `never` | Do not perform the prohibited action whenever the rule applies. | “Do not commit generated credentials.” |
| `prefer` | Use this as the default among valid options. Deviate only for a concrete constraint, stronger rule, or relevant context. Make a material deviation visible. | “Prefer small focused changes.” |
| `context` | Use this fact to understand scope or evaluate which directives apply. It does not command an action by itself. | “The release branch is maintained by the release team.” |

A narrow trigger does not make a directive `context`:

- “For release changes, run the release check” is `always`.
- “For release changes, do not edit generated manifests” is `never`.
- “For release changes, prefer the stable toolchain” is `prefer`.
- “Release changes use signed artifacts” is `context`.

### Mixed directives

Classify each enforceable clause by meaning. For example:

> Do not modify adapter code; produce an implementation handoff instead.

The prohibition belongs in `never`. The positive follow-up belongs in `always` if it is independently required. Keep both clauses in one `never` rule only when the follow-up is the required recovery from that exact prohibition and rule consumers preserve the complete text.

A `prefer` rule cannot override an applicable `always` or `never` rule. If applicable required and prohibited rules conflict, request clarification. Do not resolve the conflict by changing categories or ignoring one rule.

## Use lifecycle operations with authority

Tandem keeps these signals separate:

- workflow `state` says where active work is (`todo`, `in-progress`, `validation`);
- `accord.status` records the work agreement and delivery outcome;
- the archive outcome (`completed`, `canceled`, or `failed`) records how finished work ended.

Review status is not stored. No command sets `state` directly. Claiming moves `todo` work to `in-progress`; `accord rework` and `accord release` move work back to `in-progress` and `todo`; and `tandem review <id> --criterion <exact acceptance criterion> --note <text>` is the only route to `validation`. `accord deliver` requires `--summary` and at least one non-empty `--evidence`, sets `accord.status: delivered`, and leaves `state` unchanged. `tandem complete` accepts a delivered Accord and archives the Task in one step; completing a Task that was never delivered warns but succeeds. Consume the actual result instead of predicting or reconstructing it.

Technical capability is not authority. The fact that a command can complete a task, or that completion warnings are non-blocking, does not authorize an actor to use it. Before a lifecycle mutation, confirm that the assignment, responsible caller, or applicable workspace policy authorizes that actor and transition.

Automated checks are evidence only. Delivery can record that evidence. Evidence does not by itself accept an accord, approve a review, cancel work, or authorize completion. Keep terminal actions with the responsible actor defined by the current coordination arrangement.

## Obtain project context

Use each Tandem record for its intended purpose:

- Board documents: active scope and state.
- Parent and child relationships: strict hierarchy.
- Blockers: hard dependencies.
- References and related files: loose implementation context.
- Decisions: durable product or architecture choices.
- Rules: active repository policy.
- Logs: completed and canceled history.
- Events: audit detail, not reconstructed current state.

Search Tandem records before an ad hoc filesystem scan when the question concerns tasks, accords, reviews, Decisions, Rules, or Logs. Read raw Markdown only for inspection or repair that the Tandem implementation cannot perform.

## Record small friction without interrupting work

Use `tandem add task --kind papercut` (or an integration's thin equivalent) when small, non-blocking friction causes confusion, avoidable retries, unnecessary effort, or a workaround worth preserving. Then continue current work. A failed tool call is only a signal: expected test failures, empty searches, and deliberate invalid probes are not automatically Papercuts.

Do not use a Papercut when work is blocked; use the blocking lifecycle. Do not use one instead of a planned fix; create a Task and reference the Papercut. A thin integration maps its papercut intent to CLI argument arrays (`add task --kind papercut`, `list --kind papercut`), requests JSON for reads, and leaves parsing, IDs, validation, writes, and events to Tandem.

## Leave syncing to Tandem

In a Git repository the board syncs through the repository's `tandem` branch, separately from source commits. Tandem publishes every mutation, refreshes stale reads, and resolves routine concurrent changes itself. An agent or adapter does not stage, commit, push, or pull `.tandem/`, and source commits never include it.

- Use the ID a mutation returns. A record created while offline has a temporary `<prefix>-new-<hex>` ID until it syncs; the temporary ID keeps resolving afterwards.
- Treat `data.sync.status` as information: `pending` means the change is saved locally and will publish later. It is not a failure.
- When a result or `tandem sync status` reports a conflict or held edit, surface it to the responsible human. Resolve only with `tandem sync resolve <id> --keep local|remote|edited` when authorized. Never edit, delete, or rename `.tandem/` files to fix sync.
- A linked worktree (for example a delegated Worker's) uses the main worktree's board; it has no `.tandem/` of its own.

## Adapter boundary

A conforming adapter can:

- discover and diagnose an installed Tandem implementation;
- expose read and mutation operations in framework-native schemas;
- invoke Tandem without shell interpolation;
- request structured output for reads;
- expose papercut Tasks (`--kind papercut`) through the same CLI-only boundary;
- preserve and render Tandem results and diagnostics;
- add framework-specific approval and confirmation UX.

It must not:

- implement a second Markdown/frontmatter parser or mutation path;
- allocate IDs or reclassify relationships;
- silently initialize or upgrade a workspace;
- generate, copy, parse, or override Tandem actor identity;
- stage, commit, push, or rewrite `.tandem/` or the `tandem` branch;
- infer lifecycle authority from command availability;
- promote one repository's active rules into universal guidance;
- hide protocol warnings or contradictory instructions.

The `tandem` CLI, with `--json` for reads, is the integration surface. Adapter repositories should consume this contract and keep implementation details in their own code and maintainer documentation.
