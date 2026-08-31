---
id: decision-3
type: decision
title: "Define the Tandem project stack and tooling realms"
status: "accepted"
date: "2026-07-04"
deciders: ["Algorant"]
context: "Tandem uses several implementation, documentation, automation, deployment, and coordination tools. Some tool choices are already captured in focused decisions, but the project needs a general stack decision so future work does not drift between package managers, runners, deployment paths, or implementation languages by accident."
consequences: ["Agents and humans should default to the listed tools by realm and avoid introducing alternatives without explicit rationale.", "Bun migration tasks should bring docs/site automation into alignment with this stack decision and decision-2.", "Future stack changes should be recorded as explicit Tandem decisions or scoped task exceptions."]
alternatives: ["Leave stack choices implicit across AGENTS.md, tasks, and prior decisions; rejected because it encourages tool drift and repeated re-litigation.", "Create separate decisions for every tool immediately; rejected because a single stack map is clearer while focused decisions can still handle major changes."]
references: ["decision-1", "decision-2", "task-66", "task-89", "task-90", "task-91"]
tags: ["stack", "tooling", "docs", "rust", "typescript", "bun", "ci"]
createdAt: "2026-07-04T13:10:04Z"
updatedAt: "2026-07-04T13:10:04Z"
---

## Status

Accepted

## Context

Tandem is a local-first protocol/toolchain repository with a Rust CLI/TUI, Pi/agent integrations, an Astro/Starlight documentation site, GitHub-hosted source/deployment, and Tandem/Shep-based project coordination. The project benefits from clear tool ownership by realm so agents and humans choose the expected tools without re-litigating or accidentally mixing stacks.

This decision summarizes the current stack and cross-references narrower decisions where they exist.

## Decision

Use the following stack designations by default:

### Runtime and local tool versions

- **mise** is the preferred local runtime/tool version manager for developer machines and one-off commands when a specific runtime version is needed.
- Repository scripts should still validate required versions directly where practical so CI and non-mise environments remain understandable.

### Local automation

- **just** is the canonical repo task runner for local shortcuts such as docs preview/build, TUI development, release flows, and repeatable validation bundles.
- `justfile` recipes should reflect the same package manager and validation paths used by CI unless a documented exception exists.

### Core Tandem product

- **Rust** is the canonical implementation language for the user-facing `tandem` CLI/TUI.
- **Cargo** is the canonical Rust build/test/package tool.
- **Ratatui + crossterm** are the current TUI stack.
- For v0, keep the Rust app under `tandem/`; do not introduce a root Rust workspace or crate split without an explicit decision.

### JavaScript/TypeScript tooling

- **Bun** is the default JavaScript/TypeScript package manager, script runner, and local/CI automation tool unless a concrete incompatibility is documented after reasonable attempts.
- **TypeScript** is the canonical language for Pi extension/adapters and related JS/TS integration code under `extensions/`.
- npm-specific workflows, lockfiles, or commands require documented exception evidence per the Bun decision and rules.

### Documentation site

- **Astro + Starlight** are the canonical documentation site framework.
- Canonical Markdown source lives under `docs/`; the site project under `site/` owns rendering, navigation, theming, and static build output.
- Docs-site package management and scripts should align with the Bun decision once migration work is validated.
- Theme/design work should use Starlight-supported customization paths such as `customCss`, Expressive Code configuration, and component overrides before changing frameworks.

### Deployment, CI, and releases

- **GitHub Actions** is the canonical CI/deployment automation surface.
- **GitHub Pages** is the canonical documentation hosting target.
- Tandem CLI releases use git tags plus GitHub Release objects; a pushed tag alone is not a complete release unless explicitly requested.

### Source control and hosting

- **Git** is the canonical local version-control system.
- **GitHub** is the canonical remote hosting location for this repository.

### Project coordination and agent orchestration

- **Tandem** is the canonical durable coordination system for tasks, decisions, rules, accords, validation, and completed logs.
- **Shep/Herdr** is the preferred delegation/orchestration path for small reviewable agent work units.
- **Sideshow** is the preferred visual preview/review aid for docs, site, UI, diagrams, and product-facing mockups where useful.

## Consequences

- New work should use the stack tool for its realm by default.
- Deviations should be captured as explicit decisions, task-scoped exception notes, or rules instead of silent drift.
- CI, just recipes, and local docs should be kept aligned so validation evidence matches actual deployment behavior.
- Specific decisions such as docs framework or package manager decisions remain authoritative for their focused areas; this decision acts as the stack map tying them together.
