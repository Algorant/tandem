# Tandem CLI/TUI release checklist

## v0.5.0 (recommended tag: `tandem-v0.5.0`)

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

1. Update `GITHUB_RELEASE_NOTES.md` with version-specific highlights, user-facing changes, a dedicated bug-fix section when applicable, and any current limitations users need to know. Public release notes must not include installation commands or installation guidance.
2. Group release notes by product surface when a release includes distinct kinds of work. Prefer sections such as `Protocol`, `CLI`, `TUI`, `Docs`, and `Integrations` over a flat commit list when multiple areas changed.
3. Keep reusable validation commands, `pi-tandem` install notes, and operational checks in this checklist.
4. Optionally compare against generated notes from commits/PRs before publishing; copy only user-relevant items into the curated public notes.
5. Do not include a `Not included` section in public release notes. Readers do not have context for rejected, shelved, or never-shipped work; mention only shipped behavior and current user-facing limitations when useful.
6. Run `just release <version>`, which pushes the release commit and `tandem-v<version>` tag; the cargo-dist GitHub Actions workflow is the official binary artifact path and creates the GitHub Release with installer, archives, and checksums.
7. Verify the GitHub Release contains the expected cargo-dist assets for the supported initial targets: Linux x86_64, Linux ARM64, macOS Intel, and macOS Apple Silicon. Windows artifacts are not published initially.
8. Smoke-test the primary installer command: `curl -fsSL https://trytandem.dev/install.sh | sh`, then run `tandem --version` from the installed user-local bin directory or after updating `PATH`.
9. After the Release workflow succeeds, verify the `Update tandem-bin AUR package` workflow updates the `tandem-bin` AUR package from the published Linux x86_64 archive and `sha256.sum`. See `../docs/packaging/aur-tandem-bin.md` for secret setup, AUR remote setup, and recovery steps.

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
cargo install --git git@github.com:Algorant/tandem.git --tag tandem-v0.5.0 --path tandem --locked
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

# After the tag workflow publishes the GitHub Release:
gh release view tandem-v0.5.0 --json tagName,assets
for asset in \
  tandem-installer.sh \
  tandem-x86_64-unknown-linux-gnu.tar.xz \
  tandem-aarch64-unknown-linux-gnu.tar.xz \
  tandem-x86_64-apple-darwin.tar.xz \
  tandem-aarch64-apple-darwin.tar.xz \
  sha256.sum; do
  gh release download tandem-v0.5.0 --pattern "$asset" --dir /tmp/tandem-release-check
  test -s "/tmp/tandem-release-check/$asset"
done
grep -E 'tandem-(x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|x86_64-apple-darwin|aarch64-apple-darwin)\.tar\.xz' /tmp/tandem-release-check/sha256.sum
curl -fsSL https://trytandem.dev/install.sh | sh
tandem --version
# Verify the AUR workflow completed, or manually re-run it for tandem-v0.5.0 and confirm PKGBUILD/.SRCINFO updated tandem-bin.
```

### Release commands

```text
just release 0.5.0
```

The pushed `tandem-v0.5.0` tag triggers `.github/workflows/release.yml`, which uses cargo-dist to create the GitHub Release and upload `tandem-installer.sh`, platform archives for Linux x86_64, Linux ARM64, macOS Intel, and macOS Apple Silicon, per-artifact SHA-256 files, and `sha256.sum`. Windows artifacts are not part of the initial release target set.

If a release workflow fails before creating a GitHub Release, fix the release configuration, delete the failed remote tag, and rerun `just release 0.5.0` from the corrected commit. For example:

```text
git push origin :refs/tags/tandem-v0.5.0
git tag -d tandem-v0.5.0
just release 0.5.0
```

Do not delete or reuse the tag if a GitHub Release or published artifacts were created; publish a follow-up patch version instead.

After that workflow completes successfully, `.github/workflows/aur-tandem-bin.yml` downloads `tandem-x86_64-unknown-linux-gnu.tar.xz` and `sha256.sum`, regenerates `PKGBUILD`/`.SRCINFO` for the x86_64-only initial `tandem-bin` package, and pushes the AUR Git remote with the configured SSH key. If the AUR update needs to be retried, run the workflow manually with the same `tandem-v0.5.0` tag.
