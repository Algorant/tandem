# Tandem Extensions

This area contains agent/editor integrations for Tandem.

Current scope is intentionally narrow: integrations should adapt existing Tandem control surfaces, especially the installed `tandem` CLI, instead of reimplementing Tandem protocol parsing or mutation behavior.

## Layout

```text
extensions/
├── README.md
├── plan/
│   ├── spec.md
│   └── todo.md
└── pi-tandem/
    ├── README.md
    ├── index.ts
    ├── pi-tandem.md
    ├── plan/
    │   ├── spec.md
    │   └── todo.md
    └── tests/
        └── smoke.ts
```

## Current integrations

- `pi-tandem/` — a lightweight Pi extension that exposes `tandem_*` tools and `/tandem` diagnostics over an installed `tandem` CLI.

## Adapter principle

```text
LLM / editor agent → integration adapter → tandem CLI → .tandem workspace
```

Adapters may own:

- Pi/editor tool schemas and command registration.
- Prompt guidance and agent ergonomics.
- Output formatting, truncation, and diagnostics.
- Local smoke tests for the adapter surface.

Adapters must not own:

- Tandem protocol semantics.
- Markdown/frontmatter mutation behavior.
- Alternate task, accord, rule, decision, or log parsers beyond trivial CLI JSON output handling.

Those behaviors belong in `protocol/` and the `tandem` implementation under `tandem/`.

## Testing and promotion

Work starts as repository-local extension code and smoke tests. Global Pi config promotion is a later, explicit step after review; do not edit `~/.pi/agent` from this repo task.

See also:

- `plan/spec.md` — extension-area design
- `plan/todo.md` — extension-area todo
- `pi-tandem/README.md` — Pi extension usage and tool mapping
- `../README.md` — parent project overview
