# Tandem v0.3.0

Concise public notes for the `tandem-v0.3.0` GitHub Release. Keep reusable release validation and install checklist details in `tandem/RELEASE.md`.

## Highlights

- Added `tandem update <id>` for active task metadata edits without using workflow state transitions.
- Improved global/user theme configuration for transparent terminal workflows.
- Split public GitHub Release notes from the reusable release checklist so releases stay concise.
- Documented docs-site runtime policy: use a supported Node LTS for deploys and keep npm/package-lock workflow for now.

## Install

```text
cargo install --git git@github.com:Algorant/tandem.git --tag tandem-v0.3.0 --path tandem --locked
tandem --version
```

## Notes

- No binary artifacts are published yet; install from the git tag with Cargo.
- Mutation commands remain human-readable only; structured JSON mutation output is deferred.
- TUI visual polish remains active work; badge styling is fixed for now.
