# Tandem Extensions Spec

Status: draft

The `extensions/` area is the home for future Tandem agent/editor integrations. It is the third major child area of the monorepo alongside `protocol/` and `tandem/`. It currently contains no integrations: the `pi-tandem` Pi adapter was retired (decision-10).

## Integration surface

The `tandem` CLI is the integration surface. Reads should use `tandem --json`. Integrations call the installed binary through argument arrays (`execFile` or equivalent) and never shell-interpolate input.

```text
LLM / editor → integration → execFile("tandem", args) → .tandem files
```

## Scope

An integration added here should make Tandem easier to use from an agent or editor while keeping Tandem behavior centralized in the protocol and `tandem` CLI:

- integration-specific tool schemas and command registration;
- prompt guidance for durable Tandem coordination;
- diagnostics for missing CLIs, missing workspaces, and command failures;
- rendering of CLI output;
- smoke tests and install/test documentation.

Out of scope:

- Reimplementing Tandem Markdown/frontmatter parsing or mutation logic.
- Creating a second TypeScript Tandem protocol implementation.
- Changing the Rust package layout, adding a root workspace, schemas, fixtures, or migration tools.

## Documentation sync

When an integration changes repository scope, command names, or adapter boundaries, update the parent `README.md`, `plan/spec.md`, `plan/todo.md`, `extensions/README.md`, `extensions/plan/spec.md`, `extensions/plan/todo.md`, and the integration's own documentation.
