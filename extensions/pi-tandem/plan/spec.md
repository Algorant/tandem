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
2. `TANDEM_BIN`
3. `tandem` on `$PATH`

It runs commands with `execFile(command, args, { cwd })` and never shell-interpolates user input. Tool parameters are translated into argument arrays only.

## Tool surface

Current MVP tools:

- `tandem_status` — `tandem --help`, workspace discovery, and optional `tandem list --json` health check.
- `tandem_task` — `list`, `show`, `add`, `move`, `update`, `complete`; independently tracked subtasks are normal child task calls using `parent`, which is passed directly to Tandem so the CLI owns parent-derived/nested ID allocation and relationship classification. Deprecated inline checklist authoring is not exposed or forwarded.
- `tandem_accord` — `ready`, `claim`, `deliver`, `accept`, `rework`, `block`, `fail`.
- `tandem_log` — `list`, `show`, `search`.
- `tandem_rules` — `list`, `add`, `edit`, `delete`.
- `tandem_decision` — `list`, `show`, `add` for first-class decisions, including ADR-compatible durable records that stay `type: decision`.
- `tandem_search` — active/log search.

Read actions default `json: true` and append `--json` only where the current CLI supports it. Mutation actions do not invent structured output; they return the CLI text plus captured details. The adapter neither allocates IDs nor classifies relationships: Tandem assigns parent-derived IDs for task children (including nested children), scans active/log history for sequence continuity, retains flat IDs for generic parents, and returns allocation/collision errors. Tandem's JSON naturally supplies `parentId`/`parentRelationship` on list/search/show results and computed `subtasks` summaries when showing task parents. Existing flat-ID children remain compatible because the CLI classifies from `parentId`.

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

Guidance emphasizes using `tandem_*` tools rather than direct `.tandem` edits, creating independently tracked work as ordinary parent-linked child tasks rather than deprecated inline checklist entries, modeling epics as ordinary `type: task` + `kind: epic` parents instead of separate ADR/epic protocol behavior, recording ADR-compatible choices with `tandem_decision` rather than task lifecycle state, delivering finished work into the `validation` workflow state, preserving `review:` metadata as distinct reviewer decision state, and not accepting/completing accord work unless explicitly instructed.

## Testing

Static/smoke commands:

```text
bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts
bun extensions/pi-tandem/tests/smoke.ts
bun extensions/pi-tandem/tests/pi-runtime-smoke.ts
bun extensions/pi-tandem/tests/relationship-smoke.ts
```

`smoke.ts` performs read-only checks against this repository's `.tandem` board when the checkout has one, then creates a temporary Tandem workspace for mutating task, validation-state move, accord, rule, decision, search, complete, and log coverage. Without `TANDEM_BIN`, it first builds the current repository CLI so a stale debug binary cannot mask source changes.

`pi-runtime-smoke.ts` exercises Pi's project-local extension discovery without committing runtime state: it creates `.pi/extensions/pi-tandem/index.ts` and, when the checkout lacks one, a temporary ignored `.tandem` workspace with a completed hierarchical child, a sequence-continuing sibling, and a nested child; it starts fresh `pi --mode rpc --approve --offline` with an isolated `PI_CODING_AGENT_DIR`, verifies `/tandem` is registered from the project-local loader, runs `/tandem status`, confirms the active hierarchy count, and cleans up all temporary state.

`relationship-smoke.ts` builds the current repository Tandem CLI, rejects legacy inline authoring, passes parents through pi-tandem argument builders, and verifies hierarchical/nested IDs, completed-log sequence continuity, generic non-task parent allocation/classification, existing flat-ID child compatibility, occupied-destination collision errors, persisted relationships, and computed show/list/search fields.

Manual Pi smoke after review:

```text
TANDEM_BIN="$PWD/tandem/target/debug/tandem" pi -e ./extensions/pi-tandem/index.ts
/tandem status
```

## Future work

- Add compact custom renderers only if raw text/JSON output is too noisy.
- Add richer autocomplete or task-id pickers if Pi UI APIs prove useful.
- Promote to canonical global Pi config in a separate task after local acceptance.
- Prefer adding structured mutation output to `tandem` before adding parsing to this adapter.
