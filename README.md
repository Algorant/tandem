# Tandem

Tandem is a draft local-first protocol and toolchain for human/agent project coordination.

It takes inspiration from Brainfile's file-based task board model, but leans harder into collaborative workflows: humans and agents agree on work through **accords**, move delivered work through review, and preserve completed work in useful logs.

## Monorepo layout

```text
protocol/      Tandem protocol/specification work
tandem-tui/    Rust/Ratatui TUI planning and implementation
```

## Naming

- Product/protocol: **Tandem**
- Repository: `tandem`
- Project data directory: `.tandem/`
- Project config file: `.tandem/tandem.md`
- CLI binary: `tdm`
- TUI directory: `tandem-tui/`
- Work agreement object: `accord`
- Future prefixes: `td` / `tdm`

## Current docs

- `protocol/spec.md` — protocol draft
- `tandem-tui/spec.md` — TUI draft
