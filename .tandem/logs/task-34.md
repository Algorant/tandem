---
id: task-34
type: task
title: "Rename CLI/TUI app and command to tandem"
priority: "high"
tags: ["config", "cli", "monorepo"]
createdAt: "2026-06-28T14:51:57Z"
updatedAt: "2026-06-28T15:31:34Z"
completedAt: "2026-06-28T15:31:34Z"
completion:
  summary: "Renamed CLI/TUI app and release surface to tandem; global install and command verified."
  filesChanged: ["tandem/Cargo.toml", "tandem/src/main.rs", "README.md", "AGENTS.md", "extensions/pi-tandem/index.ts"]
  validation: "Committed and pushed 3f1677e; cargo/bun/install validation passed; user verified rename and release work"
  reviewer: "ivan"
---

## Description

Decision: the user-facing CLI/TUI app is named Tandem end-to-end. The previous `tdm` command and `tandem-tui` crate/directory naming were misunderstood/misapplied and should be removed rather than preserved as primary names.

Scope:
- Rename the Rust CLI/TUI app directory from `tandem-tui/` to `tandem/`.
- Rename the Cargo package from `tandem-tui` to `tandem`.
- Rename the installed binary from `tdm` to `tandem`.
- Keep CLI and TUI in one shared Rust codebase; the TUI remains a subcommand invoked as `tandem tui`.
- Update command examples, specs, READMEs, release/install instructions, tags, and planning docs from `tdm`/`tandem-tui` to `tandem`.
- Update integrations, especially `extensions/pi-tandem`, to locate/call `tandem` instead of `tdm` and to remove obsolete `TDM_BIN`-style naming unless an explicit compatibility shim is later requested.
- Update agent guidance/monorepo docs so future work treats `tandem/` as the canonical app crate and does not reintroduce `tdm`.

Acceptance criteria:
- `cargo run --manifest-path tandem/Cargo.toml -- init|list|tui ...` works with binary name `tandem`.
- `cargo install --path tandem --locked` installs a `tandem` command.
- Repository docs describe the three major areas as `protocol/`, `tandem/`, and `extensions/`.
- No primary user-facing docs or examples refer to `tdm`; any leftover historical mentions are either removed or clearly marked obsolete.
- `extensions/pi-tandem` shells out to `tandem` via safe argument arrays.

Out of scope for this task:
- Creating a root Rust workspace.
- Splitting CLI and TUI into separate binaries/crates.
- Maintaining a `tdm` compatibility alias unless explicitly requested later.
