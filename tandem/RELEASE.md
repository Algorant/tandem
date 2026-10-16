# Tandem CLI/TUI release checklist

## Release procedure

Package scope: the `tandem` Rust package in this directory, which builds the user-facing `tandem` binary.

`RELEASES.md` at the repository root is the canonical curated release history. cargo-dist natively discovers it and appends the selected `## X.Y.Z` section to the GitHub Release body with generated install/download information. Keep reusable validation, installation, and operational details in this checklist rather than in public release notes.

### Curated release notes workflow

1. While preparing a release, update `tandem/Cargo.toml` to `X.Y.Z` and add exactly one meaningful `## X.Y.Z` section to root `RELEASES.md`. Group user-facing notes by product surface when useful, use a dedicated `Fixed` section for fixes, and mention only shipped behavior or current user-facing limitations. Do not add installation instructions or a required rolling Unreleased section.
2. Commit the prepared version and release notes. Keep detailed task, commit, and completed-log history in Tandem; use `RELEASES.md` only for concise curated release history. A larger future release may be drafted there ahead of time.
3. Run `just release X.Y.Z` from a clean local `main` checkout. It reads the cargo-dist version from `dist-workspace.toml`, provisions and invokes that version through mise, then rejects a missing, duplicate, or empty matching release section; a requested version that disagrees with `Cargo.toml`; and a cargo-dist announcement body that omits the curated notes before publication.
4. The recipe runs the existing Rust formatting, release-profile tests/builds, dist-profile build, strict clippy, and version checks, followed by the site build, built-site link check, high-severity Bun audit, web JavaScript syntax check, and `git diff --check`. All these gates complete before creating/pushing the annotated tag and `main`.
5. After pushing, the recipe waits for the matching Release workflow, then verifies the GitHub Release is non-draft/non-prerelease and includes curated notes and required nonempty assets. Only after that verification it waits for the matching `Update tandem-bin AUR package` workflow. The AUR wait is bounded; success is reported, while a failed or timed-out AUR run is explicitly reported as a downstream packaging follow-up and does not undo verified GitHub publication.
6. Smoke-test the primary installer after publication succeeds: `curl -fsSL https://trytandem.dev/install.sh | sh`, then run `tandem --version` from the installed user-local bin directory or after updating `PATH`.

### Release cycle effort and friction audit

The 0.17.0 transcript at `/tmp/tandem-release-0.17.0.log` records command-reported stage durations, not a complete end-to-end wall-clock ledger. The table separates those observations from planning estimates; machine, cache, network, and workflow queue affect actual effort.

| Step | 0.17.0 evidence / current effort | Shorten, remove, or automate? | Rationale and risk |
| --- | --- | --- | --- |
| Prepare version and curated notes; commit on `main` | Operator effort estimate: 10–30 min, varies with release content; transcript does not time it. | Keep manual. | Human curation is intentional; automating notes risks inaccurate or noisy public history. |
| Format and optimized-profile tests | Transcript: test-profile compilation reported 13.90s; tests reported 3.85s (about 18s combined command work, excluding startup/other overhead). | Keep both. | Tests remain necessary; release-profile testing lets later release build reuse compilation. |
| Release and dist builds; strict clippy; version check | Transcript reports release build 0.06s, dist build 10.67s, clippy check 4.02s, and version command's debug build 6.35s. | Keep all existing native/dist checks; run dev-check clippy early too. | These cover distinct release/profile and lint behavior. Removing one risks late failures; early clippy shortens feedback, not total release work. |
| Site build, links, and high audit | Transcript: site build reported 1.56s and audit 0.23s; no standalone link-check duration recorded. | Keep; run before any publication. | These gates catch preventable post-tag failures. The log does not support claiming an end-to-end time saving. |
| Web syntax check and diff check | Transcript reports JavaScript bundling in 2ms; diff check has no timing. | Keep. | Very low cost; catches syntax and whitespace errors. |
| Publish and verify GitHub Release | Transcript shows repeated 3-second Actions status refreshes; no total wait recorded. Current discovery bound is 30 min, then `gh run watch` waits for completion. | Keep matching-run and asset verification. | Network/queue latency is variable; weakening identity or asset checks risks validating the wrong release. |
| Verify AUR downstream workflow | 0.17.0 record identifies successful matching workflow 38305322702, despite the stale skip message. Current wait is bounded to 5 min to find a run plus 5 min to observe completion. | Restore the check; report failure/timeout without failing a verified GitHub release. | A missing/misselected workflow can otherwise appear successful; the bounded wait avoids indefinite delay while surfacing follow-up. |

Recommended flow: run `$ just dev-check` for tests, strict release-equivalent clippy, and the Git-backed smoke check while developing; prepare/commit version and release notes; run `just release X.Y.Z`, which provisions the pinned cargo-dist and performs every local gate before tag/push; verify GitHub publication, then observe/report the matching AUR workflow. Do not rerun or hand-install cargo-dist at a different version.

Concrete bounded candidates and risks:

- **Move clippy into dev-check (implemented):** catches release-gate lints at Worker delivery. Adds the strict lint check to development validation but does not remove any release check.
- **Move existing docs build/link/audit gates before publication (implemented):** reduces risk of discovering a broken release after tagging; no claim of reduced total compute time.
- **Consume the existing cargo-dist pin via mise (implemented):** removes manual global `dist` setup while preserving one version authority. If mise or provisioning is unavailable, fail before release work with a direct setup diagnostic; no fallback installer or global configuration change.
- **Restore AUR verification with a bounded downstream report (implemented):** removes stale skip/read-only instructions. Risk: up to 10 minutes of additional wait; a failure after verified GitHub publication is an explicit packaging follow-up, not release failure.
- **Do not parallelize checks or add a benchmark/polling framework:** uncertain wall-clock benefit, extra log/resource contention, and additional maintenance are not justified by this single transcript.

### Current capabilities

- CLI commands: `--version`, `version`, `init`, `list`, `show`, `add`, `move`, `update`, `complete`, `cancel`, `search`, `papercut add|list|show|resolve`, `log list|show|search`, `accord claim|deliver|accept|rework|block|fail`, `rules list|add|edit|delete`, `decision list|show|add`, `tui`, and `web`.
- JSON read paths for supported read commands using `{ "ok": true, "data": ..., "warnings": [] }` envelopes.
- Markdown/YAML-frontmatter Tandem workspace support under `.tandem/`, with active work in `.tandem/board/`, completed logs in `.tandem/logs/`, optional lightweight friction records in `.tandem/papercuts/`, and per-actor audit events in `.tandem/events/<actor-id>.jsonl`.
- Default workflow states are `todo`, `in-progress`, and `validation`; legacy `state: review` reads are tolerated.
- Conservative state/accord synchronization for common CLI transitions.
- Ratatui/crossterm TUI with top-level Board, Logs, Rules, and Decisions tabs.
- Board Validation flow for delivered work, with action hints for approve, request changes, and complete/log flows.
- Board state subviews with task metadata, local navigation, quick-add (`a`), filter (`f`), state-move (`m`), Validation (`v`), arrangement (`b`), manual reload (`r`), inline row activation (`Enter`), optional detail pane (`Tab`), and `$EDITOR` open for selected active tasks (`e`).
- A universal sectioned `?` keybinding reference plus a global read-only Papercuts utility inbox opened with `i`.
- Event/deadline-driven rendering, idle file-change hot reload with selection preservation, and safe warning/error surfacing for reload parse/load issues.
- Completed-log browser with search filtering, visible-viewport row projection, grouped rules management prompts, and basic decision browsing/add prompts.
- Built-in `default-dark` and `verdigris` themes, user theme discovery from `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, user theme selection from `$XDG_CONFIG_HOME/tandem/config.toml` or `~/.config/tandem/config.toml`, workspace selection/overrides from `.tandem/theme.toml`, and workspace Board display settings from `.tandem/config.toml`.
- Mouse tab/click/scroll support and fixed keyboard defaults.
- Read-only `tandem web [--port <port>] [--no-open]` interface with embedded assets, loopback-only one-workspace serving, Board/Validation/details/Logs/Rules/Decisions/health views, revision polling, responsive Verdigris light/dark styling, keyboard focus support, safe Markdown, and restrictive browser security headers.
- Initial `docs/` Markdown source tree and `site/` Astro Starlight docs site with GitHub Pages workflow.

### Known limitations

- Binary artifact automation is configured through cargo-dist/GitHub Actions for Linux x86_64, Linux ARM64, macOS Intel, and macOS Apple Silicon. Windows is not supported initially.
- GitHub Releases are expected to contain `tandem-installer.sh`, one archive per supported target, per-artifact SHA-256 files, and aggregate `sha256.sum` checksums.
- The primary install command is `curl -fsSL https://trytandem.dev/install.sh | sh`; that URL should redirect to the cargo-dist generated installer for the latest GitHub Release.
- AUR binary package automation updates `tandem-bin` for x86_64 after release artifacts/checksums exist; it uses `AUR_SSH_PRIVATE_KEY` to push to AUR and never builds Tandem from source in the AUR package.
- No root Rust workspace or split crates; Cargo source install commands must target `--path tandem`.
- Most mutation commands remain human-readable; `tandem add --json` now provides structured creation output, while broader structured mutation output is deferred.
- The web interface is local and read-only. Web mutations, remote/LAN access and authentication, SSE/WebSockets, database/sync providers, multi-workspace serving, and agent feedback remain deferred.
- TUI gaps remain for configurable keymaps, Papercut mutation actions, decision reference/tag prompt parity, richer action buttons, and state/accord divergence warning surfaces.
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

### Install target for integrations

Integrations resolve `tandem` in this order:

1. `TANDEM_BIN`
2. `tandem` on `$PATH`

After the release tag exists, replace `X.Y.Z` with the exact pushed release version and install the `tandem` package from git:

```text
cargo install --git git@github.com:Algorant/tandem.git --tag tandem-vX.Y.Z tandem --locked
tandem --version
```

If installing from a local checkout before the tag is pushed, use:

```text
cargo install --path tandem --locked
tandem --version
```

To use a local build without installing globally, set an explicit binary path:

```text
TANDEM_BIN="$PWD/tandem/target/release/tandem"
```

### Release validation commands

```text
cd tandem
cargo fmt --check
cargo test --release
cargo build --release
cargo build --profile dist
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --version
cd ../site
bun install --frozen-lockfile
bun run build
bun run check:links
bun audit --audit-level=high
cd ..
node --check tandem/src/web/app.js
node --check tandem/src/web/api.js
node --check tandem/src/web/ui.js
git diff --check

# `just release X.Y.Z` performs the native, docs, and syntax gates before publication,
# then verifies the generated announcement and published GitHub assets/notes.
# It also verifies the matching AUR workflow when available; downstream AUR failure
# or timeout is reported without invalidating a verified GitHub Release.
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

After the Release workflow succeeds and the GitHub assets/notes have been verified, `.github/workflows/aur-tandem-bin.yml` downloads `tandem-x86_64-unknown-linux-gnu.tar.xz` and `sha256.sum`, regenerates `PKGBUILD`/`.SRCINFO` for the x86_64-only initial `tandem-bin` package, and pushes the AUR Git remote. `just release` verifies the matching workflow with bounded waiting; a failed or missing downstream AUR update is reported for follow-up without invalidating the verified GitHub Release. For AUR package recovery details see [`AUR.md`](AUR.md).
