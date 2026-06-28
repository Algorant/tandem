# Tandem CLI/TUI release notes

## v0.1.0 (recommended tag: `tandem-v0.1.0`)

Package scope: the `tandem` Rust package in this directory, which builds the user-facing `tandem` binary.

This is the first installable Tandem CLI/TUI release target. It is intended to unblock downstream integrations, including `pi-tandem`, with a stable `tandem` binary rather than an unreleased workspace build.

### Current capabilities

- CLI commands: `init`, `list`, `show`, `add`, `move`, `complete`, `search`, `log list|show|search`, `accord ready|claim|deliver|accept|rework|block|fail`, `rules list|add|edit|delete`, `decision list|show|add`, and `tui`.
- JSON read paths for supported read commands using `{ "ok": true, "data": ..., "warnings": [] }` envelopes.
- Markdown/YAML-frontmatter Tandem workspace support under `.tandem/`, with active work in `.tandem/board/`, completed logs in `.tandem/logs/`, and audit events in `.tandem/events.jsonl`.
- Raw-source minimal-diff mutations for task movement, completion metadata, accord lifecycle, rules, and decisions.
- Ratatui/crossterm TUI with Board, Review, Logs, Rules, and Decisions tabs.
- Board state subviews with task metadata, local navigation, quick-add (`a`), previous/next state moves (`H`/`L`), manual reload (`r`), and `$EDITOR` open for selected active tasks (`e`).
- Idle file-change hot reload with selection preservation where possible and safe warning/error surfacing for reload parse/load issues.
- Review queue, completed-log browser with search filtering, grouped rules management prompts, and basic decision browsing/add prompts.
- Built-in `default-dark` and `verdigris` themes, user theme discovery from `$XDG_CONFIG_HOME/tandem/themes/*.toml` or `~/.config/tandem/themes/*.toml`, and workspace selection/overrides from `.tandem/theme.toml`.
- Mouse tab/click/scroll support and fixed keyboard defaults.

### Known limitations

- No published artifact or GitHub release automation is defined in the repository yet.
- No root Rust workspace or split crates; install commands must target `--path tandem`.
- No `tandem --version` command yet; version confirmation is currently from `tandem/Cargo.toml`.
- Mutation commands are human-readable only; structured JSON mutation output is deferred.
- TUI gaps remain for richer Board mutations, Review action mutations/buttons, decision reference/tag prompt parity, richer action buttons, and `$EDITOR` support for decisions/custom documents.
- Keybindings are fixed defaults; custom keymap config is deferred.
- Markdown rendering is styled basics only.
- Brainfile import/migration, schemas/fixtures, MCP/hooks/auth, templates, and external archive integrations are out of scope for v0.

### Install target for `pi-tandem`

`pi-tandem` resolves `tandem` in this order:

1. `TANDEM_BIN`
2. `TANDEM_BIN`
3. `tandem` on `$PATH`

After the release tag exists, install from git with:

```text
cargo install --git git@github.com:Algorant/tandem.git --tag tandem-v0.1.0 --path tandem --locked
```

If installing from a local checkout before the tag is pushed, use:

```text
cargo install --path tandem --locked
```

For Pi smoke tests without installing globally, set an explicit binary path:

```text
TANDEM_BIN="$PWD/tandem/target/release/tandem" pi -e ./extensions/pi-tandem/index.ts
```

### Release blocker and proposed commands

This repository currently has no committed release policy, no existing release tags, and no documented credential/artifact publishing settings. Per task constraints, do not create or push the tag or GitHub release without parent/human approval.

Recommended release sequence after approval:

```text
cd tandem
cargo fmt --check
cargo test
cargo build --release
cd ..
git diff --check
git status --short
git tag -a tandem-v0.1.0 -m "Release tandem v0.1.0"
git push origin tandem-v0.1.0
# Optional, if GitHub CLI auth/release policy is approved:
gh release create tandem-v0.1.0 --title "tandem v0.1.0" --notes-file tandem/RELEASE.md
```
