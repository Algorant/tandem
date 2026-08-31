---
id: decision-3
type: decision
title: "Define the Tandem project stack and tooling realms"
status: "accepted"
deciders: ["Algorant"]
tags: ["stack", "tooling", "docs", "rust", "typescript", "bun", "ci"]
createdAt: "2026-08-31T20:52:47Z"
decidedAt: "2026-07-04T00:00:00Z"
updatedAt: "2026-08-31T20:52:47Z"
---

## Status

Accepted. Amended by the protocol 0.3.0 cutover decision, which adds clap as the canonical CLI parser.

## Context

Tandem spans a Rust CLI/TUI, Pi/agent integrations, an Astro/Starlight docs site, GitHub hosting/deployment, and Git-based coordination. Clear tool ownership by realm keeps agents and humans on the expected tools.

## Decision

- **mise** — preferred local runtime/tool version manager; scripts still validate required versions directly for non-mise environments.
- **just** — canonical repo task runner (docs, TUI dev, release flows, validation bundles).
- **Rust + Cargo** — canonical implementation language and build/test tool for the `tandem` CLI/TUI; Ratatui + crossterm for the TUI; the app stays under `tandem/` with no root workspace split.
- **clap 4.6 + derive** — canonical CLI parser (added by the protocol 0.3.0 cutover decision).
- **Bun** — default JS/TS package manager, runner, and CI automation; **TypeScript** for Pi extensions/adapters.
- **Astro + Starlight** — docs site; canonical Markdown under `docs/`, rendering under `site/`.
- **GitHub Actions + Pages** — CI and docs deployment; releases use git tags plus GitHub Release objects.
- **Git/GitHub** — version control and remote hosting.
- **Tandem** — durable coordination system; **Herdr** — delegation/orchestration path for small reviewable agent work units.

## Consequences

- Agents and humans default to the listed tools by realm.
- Future stack changes are recorded as explicit decisions or scoped exceptions.
