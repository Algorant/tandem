# Tandem Extensions

This directory is the home for future Tandem agent and editor integrations. It currently contains no integrations.

The Pi adapter (`pi-tandem`) was retired from this repository (decision-10). The `tandem` CLI, with `--json` for reads, is the integration surface: any agent or editor integration should call the installed `tandem` binary and consume its JSON output.

## Integration principle

```text
LLM / editor agent → integration adapter → installed tandem CLI → app/project → .tandem workspace
```

Integrations may own framework-specific tool schemas, prompt guidance, output rendering, and diagnostics. They must not own:

- Tandem protocol semantics.
- Markdown/frontmatter parsing or mutation.
- ID allocation or relationship reclassification.
- Alternate task, accord, rule, decision, or log parsers beyond handling CLI JSON output.

Normative behavior belongs in repository `protocol/`; executable behavior belongs in `tandem/src/protocol/` and shared `app` operations over `project::TandemProject`. Integrations use `execFile` or an equivalent argument-array API, never shell interpolation, and must not import Rust internals or bypass the CLI.

See also:

- `plan/spec.md` — extension-area design
- `plan/todo.md` — extension-area todo
- `../docs/guides/agents-and-adapters.md` — framework-neutral integration guidance
- `../README.md` — parent project overview
