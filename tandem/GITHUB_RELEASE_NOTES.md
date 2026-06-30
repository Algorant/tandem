# Tandem v0.2.3

Concise public notes for the `tandem-v0.2.3` GitHub Release. Keep this file version-specific; keep reusable release validation and install checklist details in `tandem/RELEASE.md`.

## Highlights

- Validation is now the default delivered-work workflow state, with legacy `review` reads still tolerated.
- The Rust CLI/TUI includes Board, Logs, Rules, and Decisions views plus Board Validation actions for delivered work.
- Completed logs, task/log reconciliation, accord lifecycle commands, and decision/rules commands are available through the CLI.
- The release includes initial docs-site sources and GitHub Pages build support.

## Install

```text
cargo install --git git@github.com:Algorant/tandem.git --tag tandem-v0.2.3 --path tandem --locked
tandem --version
```

## Notes

- No binary artifacts are published yet; install from the git tag with Cargo.
- Mutation commands remain human-readable only; structured JSON mutation output is deferred.
- TUI polish remains active work, especially richer Validation prompts, mouse action buttons, and warning surfaces.
