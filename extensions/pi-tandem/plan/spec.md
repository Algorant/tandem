# pi-tandem Adapter Spec

Status: MVP implementation
Date: 2026-07-10

`pi-tandem` is a Pi extension that adapts an installed `tandem` CLI into Pi tools. It follows the local `pi-web-tools` convention: a small TypeScript adapter over a CLI, not a duplicated implementation of the underlying system.

## Goals

- Let Pi agents inspect and mutate Tandem work through first-class `tandem_*` tools.
- Prefer structured `tandem --json` read paths where available.
- Preserve useful human-readable CLI output for mutation paths.
- Provide clear diagnostics when the CLI or workspace is unavailable or incompatible.
- Nudge agents toward `tandem_*` tools whenever `.tandem/tandem.md` exists or durable coordination is requested.

## Non-goals

- No direct Tandem protocol parser in TypeScript.
- No direct Markdown/frontmatter mutations.
- No global Pi config edits during local implementation.
- No root Rust workspace, schemas, fixtures, Brainfile import/migration, or new package architecture.

## Command runner

The adapter resolves the `tandem` binary in this order:

1. `TANDEM_BIN`
2. `tandem` on `$PATH`

It runs commands with `execFile(command, args, { cwd })` and never shell-interpolates user input. Tool parameters are translated into argument arrays only.

## Tool surface

Current MVP tools:

- `tandem_status` — `tandem --help`, workspace discovery, and optional `tandem list --json` health check.
- `tandem_task` — `list`, `show`, `add`, `move`, `update`, `complete`, `cancel`; update supports presence-sensitive exact Markdown `body` replacement, cancellation requires a reason and archives a canceled Log, while `kind` and `parent` pass directly to Tandem. The CLI owns canonical role resolution and allocation: Epics are root global tasks, direct Epic children are global Tasks, and direct Task children are parent-derived leaf Subtasks. Deprecated inline checklist authoring is not exposed or forwarded.
- `tandem_accord` — `claim`, `deliver`, `accept`, `rework`, `block`, `fail`. Legacy persisted `accord.status: ready` remains readable but is not an action.
- `tandem_log` — `list`, `show`, `search`.
- `tandem_rules` — `list`, `add`, `edit`, `delete`.
- `tandem_decision` — `list`, `show`, `add` for first-class decisions, including ADR-compatible durable records that stay `type: decision`.
- `tandem_search` — active/log search.

Read actions default `json: true` and append `--json` only where the current CLI supports it. Mutation actions do not invent structured output; they return the CLI text plus captured details. The adapter neither allocates IDs nor classifies relationships. Tandem assigns global IDs to Epics and Tasks (including direct Epic Tasks and decision/custom-parented Tasks), assigns `<Task ID>-M` only to Subtasks, scans active/log history for sequence continuity, validates strict leaf depth, and rejects role-changing or ID-invalidating reparenting. Tandem's JSON supplies `parentId` plus stable `epic-task`, `subtask`, or generic `parent` relationships; show supplies `tasks` for Epics and `subtasks` for Tasks. Erroneous hierarchical direct Epic children receive no compatibility exception.

## Slash command

`/tandem help|status` is registered for lightweight human diagnostics in Pi.

## Diagnostics

The adapter classifies:

- missing `tandem` executable;
- missing `.tandem/tandem.md` workspace;
- unsupported flags/subcommands from old or mismatched CLIs;
- command timeout/abort;
- generic non-zero command failures with stdout/stderr evidence.

## Prompt guidance

The extension provides:

- tool `promptSnippet` and `promptGuidelines` metadata;
- a small `before_agent_start` system-prompt addendum when a Tandem workspace is present or the prompt asks for durable coordination;
- `pi-tandem.md` as human-readable guidance for agents/config promotion.

Guidance emphasizes using `tandem_*` tools rather than direct `.tandem` edits; passing parent directly to Tandem; consuming canonical Epic → global Task → parent-derived leaf Subtask output; and never reclassifying relationships in TypeScript. Only Tasks are initial delegation roots: one Task worker owns its Subtasks through the todo projection, while Epics and Subtasks are not delegated. Guidance also preserves lifecycle/review/accord separation and ADR-compatible decisions.

## Testing

Static/smoke commands:

```text
bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts
bun extensions/pi-tandem/tests/smoke.ts
bun extensions/pi-tandem/tests/pi-runtime-smoke.ts
bun extensions/pi-tandem/tests/relationship-smoke.ts
```

`smoke.ts` performs read-only checks against this repository's `.tandem` board when the checkout has one, then creates a temporary Tandem workspace for mutating task, validation-state move, accord, rule, decision, search, complete, and log coverage. Without `TANDEM_BIN`, it first builds the current repository CLI so a stale debug binary cannot mask source changes.

`pi-runtime-smoke.ts` exercises Pi's project-local extension discovery without committing runtime state: it creates `.pi/extensions/pi-tandem/index.ts` and, when the checkout lacks one, a temporary ignored `.tandem` workspace with an Epic, global Task, completed Subtask, and sequence-continuing active Subtask. It verifies CLI-returned `epic-task`/`subtask` output before fresh Pi startup, then confirms `/tandem` registration/status and cleans up all temporary state.

`relationship-smoke.ts` builds the current repository Tandem CLI, asserts generated Task-only/thin-adapter guidance, rejects legacy inline authoring, passes kind/parent through pi-tandem argument builders, and verifies Epic → global Task → parent-derived Subtask allocation; generic parents; Board+Logs `tasks`/`subtasks` show collections; `epic-task`, `subtask`, and generic `parent` output; completed-log continuity; exact-parent reads; and rejection of nested Epics, children beneath Subtasks, role-changing reparenting, erroneous hierarchical Epic children, and erroneous global-ID Subtasks.

Manual Pi smoke after review:

```text
TANDEM_BIN="$PWD/tandem/target/debug/tandem" pi -e ./extensions/pi-tandem/index.ts
/tandem status
```

## Cross-repository handoff

The worker/delegation implementation belongs in canonical Pi config, not this repository. Follow [`../../../plan/delegated-task-tree-worker-spec.md`](../../../plan/delegated-task-tree-worker-spec.md), which defines the explicit Pi-config handoff. Tandem repository work must not modify personal dotfiles.

## Future work

- Add compact custom renderers only if raw text/JSON output is too noisy.
- Add richer autocomplete or task-id pickers if Pi UI APIs prove useful.
- Promote to canonical global Pi config in a separate task after local acceptance.
- Prefer adding structured mutation output to `tandem` before adding parsing to this adapter.
