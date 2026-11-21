# Tandem CLI/TUI release checklist

## Release procedure

Package scope: the `tandem` Rust package in this directory, which builds the user-facing `tandem` binary.

`RELEASES.md` at the repository root is the canonical curated release history. cargo-dist natively discovers it and appends the selected `## X.Y.Z` section to the GitHub Release body with generated install/download information. Keep reusable validation, installation, and operational details in this checklist rather than in public release notes.

### Curated release notes workflow

1. While preparing a release, update `tandem/Cargo.toml` to `X.Y.Z` and add exactly one meaningful `## X.Y.Z` section to root `RELEASES.md`. Group user-facing notes by product surface when useful, use a dedicated `Fixed` section for fixes, and mention only shipped behavior or current user-facing limitations. Do not add installation instructions or a required rolling Unreleased section.
2. Commit the prepared version and release notes. Keep detailed task, commit, and completed-log history in Tandem; use `RELEASES.md` only for concise curated release history. A larger future release may be drafted there ahead of time.
3. Run `just release X.Y.Z` from a clean local `main` checkout. It reads the cargo-dist version from `dist-workspace.toml`, provisions and invokes that version through mise, then rejects a missing, duplicate, or empty matching release section; a requested version that disagrees with `Cargo.toml`; and a cargo-dist announcement body that omits the curated notes before publication.
4. The recipe runs the existing Rust formatting, release-profile tests/builds, dist-profile build, strict clippy, and version checks, followed by the site build, built-site link check, high-severity Bun audit, and `git diff --check`. All these gates complete before creating/pushing the annotated tag and `main`.
5. After pushing, the recipe waits for the matching Release workflow, then verifies the GitHub Release is non-draft/non-prerelease and includes curated notes and required nonempty assets. GitHub publication is the end of the release; AUR and other downstream packaging are not part of it.
6. Smoke-test the primary installer after publication succeeds: `curl -fsSL https://trytandem.dev/install.sh | sh`, then run `tandem --version` from the installed user-local bin directory or after updating `PATH`.
7. For mise users picking up the just-published release immediately, run `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem`, then confirm `tandem --version` reports the exact published version; do not trust `latest` while mise's versions host may be stale.

### Release cycle effort and friction audit

The 0.17.0 transcript at `/tmp/tandem-release-0.17.0.log` records command-reported stage durations, not a complete end-to-end wall-clock ledger. The table separates those observations from planning estimates; machine, cache, network, and workflow queue affect actual effort.

| Step | 0.17.0 evidence / current effort | Shorten, remove, or automate? | Rationale and risk |
| --- | --- | --- | --- |
| Prepare version and curated notes; commit on `main` | Operator effort estimate: 10–30 min, varies with release content; transcript does not time it. | Keep manual. | Human curation is intentional; automating notes risks inaccurate or noisy public history. |
| Format and optimized-profile tests | Transcript: test-profile compilation reported 13.90s; tests reported 3.85s (about 18s combined command work, excluding startup/other overhead). | Keep both. | Tests remain necessary; release-profile testing lets later release build reuse compilation. |
| Release and dist builds; strict clippy; version check | Transcript reports release build 0.06s, dist build 10.67s, clippy check 4.02s, and version command's debug build 6.35s. | Keep all existing native/dist checks; run dev-check clippy early too. | These cover distinct release/profile and lint behavior. Removing one risks late failures; early clippy shortens feedback, not total release work. |
| Site build, links, and high audit | Transcript: site build reported 1.56s and audit 0.23s; no standalone link-check duration recorded. | Keep; run before any publication. | These gates catch preventable post-tag failures. The log does not support claiming an end-to-end time saving. |
| Diff check | Transcript records no timing. | Keep. | Very low cost; catches whitespace errors. |
| Publish and verify GitHub Release | Transcript shows repeated 3-second Actions status refreshes; no total wait recorded. Current discovery bound is 30 min, then `gh run watch` waits for completion. | Keep matching-run and asset verification. | Network/queue latency is variable; weakening identity or asset checks risks validating the wrong release. |

Recommended flow: run `$ just dev-check` for tests, strict release-equivalent clippy, and the Git-backed smoke check while developing; prepare/commit version and release notes; run `just release X.Y.Z`, which provisions the pinned cargo-dist and performs every local gate before tag/push; verify GitHub publication. Do not rerun or hand-install cargo-dist at a different version.

Concrete bounded candidates and risks:

- **Move clippy into dev-check (implemented):** catches release-gate lints at Worker delivery. Adds the strict lint check to development validation but does not remove any release check.
- **Move existing docs build/link/audit gates before publication (implemented):** reduces risk of discovering a broken release after tagging; no claim of reduced total compute time.
- **Consume the existing cargo-dist pin via mise (implemented):** removes manual global `dist` setup while preserving one version authority. If mise or provisioning is unavailable, fail before release work with a direct setup diagnostic; no fallback installer or global configuration change.
- **Remove AUR automation (implemented):** the AUR package was unused and its job broke on every Arch rolling change; dropping it removes a recurring failure with no effect on GitHub publication.
- **Do not parallelize checks or add a benchmark/polling framework:** uncertain wall-clock benefit, extra log/resource contention, and additional maintenance are not justified by this single transcript.

### Current capabilities

The maintained CLI command tree and JSON contract are documented by [`tandem --help`](../docs/cli/index.md) and the [CLI reference](../docs/cli/index.md); the normative workspace layout and protocol are in [`protocol/README.md`](../protocol/README.md). The terminal UI and its current key reference are described in the [TUI guide](../docs/tui/index.md), and theme and Board display settings in [Configuration](../docs/reference/configuration.md). These sources are authoritative; this checklist intentionally avoids duplicating command, file-layout, and keybinding inventories.

### Known limitations

- Binary artifacts currently target Linux x86_64, Linux ARM64, macOS Intel, and macOS Apple Silicon; Windows is not supported.
- Tandem has no web viewer or HTTP interface; use the TUI or CLI.
- The project remains a single Rust binary crate under `tandem/`, not a root Cargo workspace or split crates; Cargo source installs target `--path tandem`.
- TUI keybindings are fixed defaults, and Markdown rendering supports styled basics only. See [Configuration](../docs/reference/configuration.md) for current settings.
- Brainfile import/migration, schemas/fixtures, MCP/hooks/auth, templates, and external archive integrations are out of scope.

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
git diff --check

# `just release X.Y.Z` performs the native and docs gates before publication,
# then verifies the generated announcement and published GitHub assets/notes.
#
# For an independent post-publication inspection, substitute the actual tag:
tag=tandem-vX.Y.Z
gh release view "$tag" --json isDraft,isPrerelease,body,assets
gh run list --workflow Release --limit 20
curl -fsSL https://trytandem.dev/install.sh | sh
tandem --version
```

### Release commands

```text
just release X.Y.Z
```

The pushed `tandem-vX.Y.Z` tag triggers `.github/workflows/release.yml`, which uses cargo-dist to create the GitHub Release and upload `tandem-installer.sh`, platform archives for Linux x86_64, Linux ARM64, macOS Intel, and macOS Apple Silicon, per-artifact SHA-256 files, and `sha256.sum`. Windows artifacts are not part of the initial release target set.

If a release workflow fails before creating a GitHub Release, fix the release configuration, delete the failed remote tag, and rerun `just release X.Y.Z` from the corrected commit. Do not delete or reuse a tag if a GitHub Release or published artifacts were created; publish a follow-up patch version instead.

Tandem no longer publishes to the AUR. The x86_64 `tandem-bin` package that was generated from release artifacts stays on the AUR frozen at 0.19.0 and is not updated; install with the shell script or mise instead.
