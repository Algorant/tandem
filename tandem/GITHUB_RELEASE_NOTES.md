# Tandem v0.4.1

Concise public notes for the `tandem-v0.4.1` GitHub Release. Keep reusable release validation and install checklist details in `tandem/RELEASE.md`.

## Highlights

- Fixed concurrent task and decision ID allocation so simultaneous writers do not overwrite newly created documents.
- Improved the TUI Board’s Epic arrangement filtering and toggle ergonomics.
- Improved inline validation previews in the TUI Board.
- Launched a more polished documentation site at `trytandem.dev` with a quickstart, custom branding, and link checks.
- Migrated docs-site automation from npm to Bun.

## CLI

- Fixed sequential ID allocation for newly created task and decision documents by reserving files atomically and retrying on collisions.
- Added regression coverage for concurrent task creation to prevent document overwrite races.

## TUI

- Changed the Board arrangement toggle from `E` to `b` and updated footer/help text to describe switching between State Board and Epic Board.
- Fixed Epic Board filtering so the Epic arrangement shows only epic groups and matching child tasks, rather than leaking unrelated unparented/orphan rows.
- Improved inline row preview sizing and validation preview rendering for Board rows.
- Included user and workspace config fingerprints in TUI reload detection so Board display settings refresh more reliably.

## Docs

- Added a quickstart guide and expanded first-pass documentation across concepts, CLI, TUI, guides, and homepage content.
- Added a docs-site workflow guide and theme tester guide.
- Added custom Tandem branding, favicon, social card metadata, and Verdigris styling for the Starlight docs site.
- Configured the docs site for the custom domain `trytandem.dev`.
- Added built-docs link checking to the docs workflow.

## Tooling

- Migrated docs-site dependency management and scripts from npm/package-lock to Bun/bun.lock.
- Updated the GitHub Pages workflow to install with Bun, run the docs build, and check generated links.
- Updated release/docs helper commands to use Bun by default.

## Install

```text
cargo install --git git@github.com:Algorant/tandem.git --tag tandem-v0.4.1 --path tandem --locked
tandem --version
```

## Notes

- No binary artifacts are published yet; install from the git tag with Cargo.
- Mutation commands remain human-readable only; structured JSON mutation output is deferred.
- TUI visual polish remains active work.
