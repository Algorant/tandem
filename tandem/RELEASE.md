# Tandem CLI/TUI release checklist

## Release procedure

Package scope: the `tandem` Rust package in this directory, which builds the user-facing `tandem` binary.

`RELEASES.md` at the repository root is the canonical curated release history. cargo-dist natively discovers it and appends the selected `## X.Y.Z` section to the GitHub Release body with generated install/download information. Keep reusable validation, installation, and operational details in this checklist rather than in public release notes.

### Curated release notes workflow

1. While preparing a release, update `tandem/Cargo.toml` to `X.Y.Z` and add exactly one meaningful `## X.Y.Z` section to root `RELEASES.md`. Group user-facing notes by product surface when useful, use a dedicated `Fixed` section for fixes, and mention only shipped behavior or current user-facing limitations. Do not add installation instructions or a required rolling Unreleased section.
2. Commit the prepared version and release notes. Keep detailed task, commit, and completed-log history in Tandem; use `RELEASES.md` only for concise curated release history. A larger future release may be drafted there ahead of time.
3. Run `just release X.Y.Z` from a clean local `main` checkout. It rejects a missing, duplicate, or empty matching release section; a requested version that disagrees with `Cargo.toml`; and a cargo-dist announcement body that omits the curated notes before it creates the tag.
4. `just release` then runs the repository validation suite, pushes `main` and `tandem-vX.Y.Z`, waits for the Release workflow, and checks the published non-draft/non-prerelease GitHub Release body and assets. It waits for the `Update tandem-bin AUR package` workflow using that release commit, its `workflow_run` event, and the Release workflow completion time, then requires it to succeed.
5. Smoke-test the primary installer command after automated publication succeeds: `curl -fsSL https://trytandem.dev/install.sh | sh`, then run `tandem --version` from the installed user-local bin directory or after updating `PATH`.

### Current capabilities

- CLI commands: `--version`, `version`, `init`, `list`, `show`, `add`, `move`, `update`, `complete`, `cancel`, `search`, `log list|show|search`, `accord claim|deliver|accept|rework|block|fail`, `rules list|add|edit|delete`, `decision list|show|add`, and `tui`.
- JSON read paths for supported read commands using `{ "ok": true, "data": ..., "warnings": [] }` envelopes.
- Markdown/YAML-frontmatter Tandem workspace support under `.tandem/`, with active work in `.tandem/board/`, completed logs in `.tandem/logs/`, and audit events in `.tandem/events.jsonl`.
- Default workflow states are `todo`, `in-progress`, and `validation`; legacy `state: review` reads are tolerated.
- Conservative state/accord synchronization for common CLI transitions.
- Ratatui/crossterm TUI with top-level Board, Logs, Rules, and Decisions tabs.
- Board Validation flow for delivered work, with action hints for approve, request changes, and complete/log flows.
- Board state subviews with task metadata, local navigation, quick-add (`a`), previous/next state moves (`H`/`L`), manual reload (`r`), inline expanded row previews (`Enter`), optional detail pane (`Tab`), and `$EDITOR` open for selected active tasks (`e`).
- Idle file-change hot reload with selection preservation where possible and safe warning/error surfacing for reload parse/load issues.
- Completed-log browser with search filtering, grouped rules management prompts, and basic decision browsing/add prompts.
- Built-in `default-dark` and `verdigris` themes, user theme discovery from `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, user theme selection from `$XDG_CONFIG_HOME/tandem/config.toml` or `~/.config/tandem/config.toml`, workspace selection/overrides from `.tandem/theme.toml`, and workspace Board display settings from `.tandem/config.toml`.
- Mouse tab/click/scroll support and fixed keyboard defaults.
- Initial `docs/` Markdown source tree and `site/` Astro Starlight docs site with GitHub Pages workflow.

### Known limitations

- Binary artifact automation is configured through cargo-dist/GitHub Actions for Linux x86_64, Linux ARM64, macOS Intel, and macOS Apple Silicon. Windows is not supported initially.
- GitHub Releases are expected to contain `tandem-installer.sh`, one archive per supported target, per-artifact SHA-256 files, and aggregate `sha256.sum` checksums.
- The primary install command is `curl -fsSL https://trytandem.dev/install.sh | sh`; that URL should redirect to the cargo-dist generated installer for the latest GitHub Release.
- AUR binary package automation updates `tandem-bin` for x86_64 after release artifacts/checksums exist; it uses `AUR_SSH_PRIVATE_KEY` to push to AUR and never builds Tandem from source in the AUR package.
- No root Rust workspace or split crates; Cargo source install commands must target `--path tandem`.
- Most mutation commands remain human-readable; `tandem add --json` now provides structured creation output, while broader structured mutation output is deferred.
- TUI gaps remain for richer Board mutations, richer Validation mutation prompts, mouse action buttons, keybinding/help final polish, decision reference/tag prompt parity, and state/accord divergence warning surfaces.
- Keybindings are fixed defaults; custom keymap config is deferred.
- Markdown rendering is styled basics only.
- Brainfile import/migration, schemas/fixtures, MCP/hooks/auth, templates, and external archive integrations are out of scope for v0.
- Docs-site build currently succeeds but may emit a Starlight warning about `Entry docs → 404 was not found`; this is non-blocking for the generated static output and should be tracked as docs-site polish.

### Install target

The primary user-facing install command is the branded installer URL:

```text
curl -fsSL https://trytandem.dev/install.sh | sh
tandem --version
```

`https://trytandem.dev/install.sh` must be maintained as a provider-level HTTP redirect to the cargo-dist generated installer on the latest GitHub Release:

```text
https://github.com/Algorant/tandem/releases/latest/download/tandem-installer.sh
```

GitHub Pages cannot express this redirect from the repository, and `site/public/install.sh` must not be restored as a shell wrapper. Keep OS/architecture detection, release asset selection, checksums, and install behavior in cargo-dist's generated installer. The install should remain user-local/no-sudo. If users cannot run `tandem` after install, direct them to add the reported cargo-dist bin directory, commonly `~/.local/bin` or `~/.cargo/bin`, to `PATH`.

### Install target for `pi-tandem`

`pi-tandem` resolves `tandem` in this order:

1. `TANDEM_BIN`
2. `tandem` on `$PATH`

After the release tag exists, install from git with:

```text
cargo install --git git@github.com:Algorant/tandem.git --tag tandem-v0.6.5 --path tandem --locked
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
bun install --frozen-lockfile
bun run build
bun run check:links
bun audit --audit-level=high
cd ..
bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts
TANDEM_BIN="$PWD/tandem/target/release/tandem" bun extensions/pi-tandem/tests/smoke.ts
TANDEM_BIN="$PWD/tandem/target/release/tandem" bun extensions/pi-tandem/tests/relationship-smoke.ts
TANDEM_BIN="$PWD/tandem/target/release/tandem" bun extensions/pi-tandem/tests/pi-runtime-smoke.ts
git diff --check

# `just release X.Y.Z` performs these checks before reporting success:
# - cargo-dist's generated announcement body contains the matching RELEASES.md section
# - the published GitHub Release is not draft/prerelease and has that section plus all expected assets
# - the Update tandem-bin AUR package workflow succeeds
#
# For an independent post-publication inspection, substitute the actual tag:
tag=tandem-vX.Y.Z
gh release view "$tag" --json isDraft,isPrerelease,body,assets
gh run list --workflow Release --limit 20
gh run list --workflow "Update tandem-bin AUR package" --limit 20
curl -fsSL https://trytandem.dev/install.sh | sh
tandem --version
```

### Release commands

```text
just release X.Y.Z
```

The pushed `tandem-vX.Y.Z` tag triggers `.github/workflows/release.yml`, which uses cargo-dist to create the GitHub Release and upload `tandem-installer.sh`, platform archives for Linux x86_64, Linux ARM64, macOS Intel, and macOS Apple Silicon, per-artifact SHA-256 files, and `sha256.sum`. Windows artifacts are not part of the initial release target set.

If a release workflow fails before creating a GitHub Release, fix the release configuration, delete the failed remote tag, and rerun `just release X.Y.Z` from the corrected commit. Do not delete or reuse a tag if a GitHub Release or published artifacts were created; publish a follow-up patch version instead.

After the Release workflow succeeds, `.github/workflows/aur-tandem-bin.yml` downloads `tandem-x86_64-unknown-linux-gnu.tar.xz` and `sha256.sum`, regenerates `PKGBUILD`/`.SRCINFO` for the x86_64-only initial `tandem-bin` package, and pushes the AUR Git remote. `just release` treats failure or absence of this workflow as a failed release; for recovery details see `../docs/packaging/aur-tandem-bin.md`.
