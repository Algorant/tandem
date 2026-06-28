# Tandem Extensions Spec

Status: draft
Date: 2026-06-28

The `extensions/` area is the home for Tandem agent/editor integrations. It is the third major child area of the monorepo alongside `protocol/` and `tandem-tui/`.

## Scope

Extension work should make Tandem easier to use from agents and editors while keeping Tandem behavior centralized in the protocol and `tdm` CLI.

Current area responsibilities:

- Integration-specific tool schemas and command registration.
- Agent prompt guidance for durable Tandem coordination.
- Diagnostics for missing CLIs, missing workspaces, unsupported flags, and command failures.
- Human/LLM-friendly rendering of CLI output.
- Local adapter smoke tests.
- Documentation for installing and testing integrations.

Out of scope for this area:

- Reimplementing Tandem Markdown/frontmatter parsing or mutation logic.
- Creating a second TypeScript Tandem protocol implementation.
- Changing the Rust package layout, adding a root workspace, schemas, fixtures, or migration tools.
- Promoting code into global Pi config without explicit review.

## Adapter architecture

The default architecture is:

```text
LLM / Pi / editor → extension adapter → execFile("tdm", args) → .tandem files
```

Adapters should use argument arrays via `execFile` or equivalent APIs and must not shell-interpolate user input. Read paths should prefer `tdm --json` where the CLI supports it. Mutation paths may preserve human-readable CLI output until `tdm` exposes structured mutation results.

## Current adapter: pi-tandem

`extensions/pi-tandem/` provides a Pi extension modelled after the local `pi-web-tools` style:

- no provider logic and no protocol mutation logic in TypeScript;
- one small command runner around installed `tdm`;
- Pi tool registrations for tasks, accords, logs, rules, decisions, search, and status;
- `/tandem help|status` diagnostics;
- prompt guidance that prefers `tdm_*` tools when a `.tandem/tandem.md` workspace exists or durable coordination is requested.

## Testing strategy

1. Build or install `tdm`.
2. Run adapter static checks, currently `bun --check extensions/pi-tandem/index.ts`.
3. Run local smoke tests, currently `bun extensions/pi-tandem/tests/smoke.ts`.
4. Project-local Pi loading can be tested later by copying/symlinking to `.pi/extensions/pi-tandem/index.ts` or by launching Pi with `-e extensions/pi-tandem/index.ts`.
5. Global canonical Pi config promotion belongs to a later task after local review.

## Documentation sync

When an integration changes repository scope, command names, adapter boundaries, or promotion policy, update:

- parent `README.md`, `plan/spec.md`, `plan/todo.md`, and `AGENTS.md`;
- `extensions/README.md`, `extensions/plan/spec.md`, and `extensions/plan/todo.md`;
- the affected integration README/spec/todo.
