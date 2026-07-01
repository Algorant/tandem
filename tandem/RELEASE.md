# Tandem CLI/TUI release checklist

## v0.3.0 (recommended tag: `tandem-v0.3.0`)

Package scope: the `tandem` Rust package in this directory, which builds the user-facing `tandem` binary.

This file is the reusable release checklist and validation record. Do not use it directly as the GitHub Release body. Public GitHub Release notes live in `GITHUB_RELEASE_NOTES.md` so each release can stay concise and version-specific while this checklist keeps install, validation, and operational details available.

### Public GitHub Release notes workflow

Recommendation: maintain curated, per-release public notes in `tandem/GITHUB_RELEASE_NOTES.md`, and maintain reusable validation/install procedure details in this checklist.

| Option | Fit for Tandem |
| --- | --- |
| Curated per-release notes | Best default. Produces concise, useful highlights and known limitations, and lets the release owner group work by user impact instead of commit order. |
| Generated changelog from commits/tags | Useful as an internal drafting aid, but noisy unless commit hygiene and grouping are consistently release-note quality. |
| GitHub auto-generated release notes (`gh release create --generate-notes`) | Useful as a compare/draft source and can be configured with `.github/release.yml`, but should be reviewed before publishing because PR labels/titles may not explain CLI/TUI user impact. |
| Reusable checklist as release body | Avoid. It preserves validation detail, but makes public releases verbose and repeats boilerplate. Keep it in this file instead. |

Release flow:

1. Update `GITHUB_RELEASE_NOTES.md` with version-specific highlights, user-facing changes, install command, and any current limitations users need to know.
2. Group release notes by product surface when a release includes distinct kinds of work. Prefer sections such as `Protocol`, `CLI`, `TUI`, `Docs`, and `Integrations` over a flat commit list when multiple areas changed.
3. Keep reusable validation commands, `pi-tandem` install notes, and operational checks in this checklist.
4. Optionally compare against generated notes from commits/PRs before publishing; copy only user-relevant items into the curated public notes.
5. Do not include a `Not included` section in public release notes. Readers do not have context for rejected, shelved, or never-shipped work; mention only shipped behavior and current user-facing limitations when useful.
6. Run `just release <version>`, which publishes the GitHub Release from `GITHUB_RELEASE_NOTES.md`.

### Current capabilities

- CLI commands: `--version`, `version`, `init`, `list`, `show`, `add`, `move`, `complete`, `search`, `log list|show|search`, `accord ready|claim|deliver|accept|rework|block|fail`, `rules list|add|edit|delete`, `decision list|show|add`, and `tui`.
- JSON read paths for supported read commands using `{ "ok": true, "data": ..., "warnings": [] }` envelopes.
- Markdown/YAML-frontmatter Tandem workspace support under `.tandem/`, with active work in `.tandem/board/`, completed logs in `.tandem/logs/`, and audit events in `.tandem/events.jsonl`.
- Default workflow states are `todo`, `in-progress`, and `validation`; legacy `state: review` reads are tolerated.
- Conservative state/accord synchronization for common CLI transitions.
- Ratatui/crossterm TUI with top-level Board, Logs, Rules, and Decisions tabs.
- Board Validation flow for delivered work, with action hints for approve, request changes, and complete/log flows.
- Board state subviews with task metadata, local navigation, quick-add (`a`), previous/next state moves (`H`/`L`), manual reload (`r`), inline expanded row previews (`Enter`), optional detail pane (`Tab`), and `$EDITOR` open for selected active tasks (`e`).
- Idle file-change hot reload with selection preservation where possible and safe warning/error surfacing for reload parse/load issues.
- Completed-log browser with search filtering, grouped rules management prompts, and basic decision browsing/add prompts.
- Built-in `default-dark` and `verdigris` themes, user theme discovery from `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, user theme selection from `$XDG_CONFIG_HOME/tandem/config.toml` or `~/.config/tandem/config.toml`, and workspace selection/overrides from `.tandem/theme.toml`.
- Mouse tab/click/scroll support and fixed keyboard defaults.
- Initial `docs/` Markdown source tree and `site/` Astro Starlight docs site with GitHub Pages workflow.

### Known limitations

- No binary artifacts are published; install from the git tag with Cargo.
- No root Rust workspace or split crates; install commands must target `--path tandem`.
- Mutation commands are human-readable only; structured JSON mutation output is deferred.
- TUI gaps remain for richer Board mutations, richer Validation mutation prompts, mouse action buttons, keybinding/help final polish, decision reference/tag prompt parity, and state/accord divergence warning surfaces.
- Keybindings are fixed defaults; custom keymap config is deferred.
- Markdown rendering is styled basics only.
- Brainfile import/migration, schemas/fixtures, MCP/hooks/auth, templates, and external archive integrations are out of scope for v0.
- Docs-site build currently succeeds but may emit a Starlight warning about `Entry docs → 404 was not found`; this is non-blocking for the generated static output and should be tracked as docs-site polish.

### Install target for `pi-tandem`

`pi-tandem` resolves `tandem` in this order:

1. `TANDEM_BIN`
2. `tandem` on `$PATH`

After the release tag exists, install from git with:

```text
cargo install --git git@github.com:Algorant/tandem.git --tag tandem-v0.3.0 --path tandem --locked
tandem --version
```

If installing from a local checkout before the tag is pushed, use:

```text
cargo install --path tandem --locked
tandem --version
```

For Pi smoke tests without installing globally, set an explicit binary path:

```text
TANDEM_BIN="$PWD/tandem/target/release/tandem" pi -e ./extensions/pi-tandem/index.ts
```

### Release validation commands

```text
cd tandem
cargo fmt --check
cargo test
cargo build --release
cargo run -- --version
cargo run -- version
cd ../site
npm ci
npm run build
npm audit --audit-level=high
cd ..
bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts
TANDEM_BIN="$PWD/tandem/target/release/tandem" bun extensions/pi-tandem/tests/smoke.ts
TANDEM_BIN="$PWD/tandem/target/release/tandem" bun extensions/pi-tandem/tests/relationship-smoke.ts
TANDEM_BIN="$PWD/tandem/target/release/tandem" bun extensions/pi-tandem/tests/pi-runtime-smoke.ts
git diff --check
```

### Release commands

```text
git tag -a tandem-v0.3.0 -m "Release tandem v0.2.2"
git push origin main
git push origin tandem-v0.3.0
```
