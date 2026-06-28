# pi-tandem Adapter Spec

Status: MVP implementation
Date: 2026-06-28

`pi-tandem` is a Pi extension that adapts an installed `tdm` CLI into Pi tools. It follows the local `pi-web-tools` convention: a small TypeScript adapter over a CLI, not a duplicated implementation of the underlying system.

## Goals

- Let Pi agents inspect and mutate Tandem work through first-class `tdm_*` tools.
- Prefer structured `tdm --json` read paths where available.
- Preserve useful human-readable CLI output for mutation paths.
- Provide clear diagnostics when the CLI or workspace is unavailable or incompatible.
- Nudge agents toward `tdm_*` tools whenever `.tandem/tandem.md` exists or durable coordination is requested.

## Non-goals

- No direct Tandem protocol parser in TypeScript.
- No direct Markdown/frontmatter mutations.
- No global Pi config edits during local implementation.
- No root Rust workspace, schemas, fixtures, Brainfile import/migration, or new package architecture.

## Command runner

The adapter resolves the `tdm` binary in this order:

1. `TANDEM_TDM_BIN`
2. `TDM_BIN`
3. `tdm` on `$PATH`

It runs commands with `execFile(command, args, { cwd })` and never shell-interpolates user input. Tool parameters are translated into argument arrays only.

## Tool surface

Current MVP tools:

- `tdm_status` — `tdm --help`, workspace discovery, and optional `tdm list --json` health check.
- `tdm_task` — `list`, `show`, `add`, `move`, `complete`.
- `tdm_accord` — `ready`, `claim`, `deliver`, `accept`, `rework`, `block`, `fail`.
- `tdm_log` — `list`, `show`, `search`.
- `tdm_rules` — `list`, `add`, `edit`, `delete`.
- `tdm_decision` — `list`, `show`, `add`.
- `tdm_search` — active/log search.

Read actions default `json: true` and append `--json` only where the current CLI supports it. Mutation actions do not invent structured output; they return the CLI text plus captured details.

## Slash command

`/tandem help|status` is registered for lightweight human diagnostics in Pi.

## Diagnostics

The adapter classifies:

- missing `tdm` executable;
- missing `.tandem/tandem.md` workspace;
- unsupported flags/subcommands from old or mismatched CLIs;
- command timeout/abort;
- generic non-zero command failures with stdout/stderr evidence.

## Prompt guidance

The extension provides:

- tool `promptSnippet` and `promptGuidelines` metadata;
- a small `before_agent_start` system-prompt addendum when a Tandem workspace is present or the prompt asks for durable coordination;
- `pi-tandem.md` as human-readable guidance for agents/config promotion.

Guidance emphasizes using `tdm_*` tools rather than direct `.tandem` edits, and warns agents not to accept/complete accord work unless explicitly instructed.

## Testing

Static/smoke commands:

```text
bun --check extensions/pi-tandem/index.ts
bun extensions/pi-tandem/tests/smoke.ts
```

The smoke test creates a temporary Tandem workspace and exercises the wrapper argument builders against the real `tdm` CLI. If no `TANDEM_TDM_BIN`/`TDM_BIN` is set and the local debug binary is missing, it builds `tandem-tui` first.

Manual Pi smoke after review:

```text
pi -e extensions/pi-tandem/index.ts
/tandem status
```

## Future work

- Add compact custom renderers only if raw text/JSON output is too noisy.
- Add richer autocomplete or task-id pickers if Pi UI APIs prove useful.
- Promote to canonical global Pi config in a separate task after local acceptance.
- Prefer adding structured mutation output to `tdm` before adding parsing to this adapter.
