# Tandem development shortcuts

set positional-arguments

# Run the development TUI against this repository's .tandem workspace.
dev:
	cargo run --manifest-path tandem/Cargo.toml -- tui

# Start the local documentation site with Astro Starlight.
# The npm dev script syncs ../docs/ into Starlight before serving.
site:
	#!/usr/bin/env bash
	set -euo pipefail
	cd site
	if [[ ! -d node_modules ]]; then
		npm install
	fi
	npm run dev

# Alias for the local documentation site.
alias docs := site

# Build the static documentation site output in site/dist/.
site-build:
	#!/usr/bin/env bash
	set -euo pipefail
	cd site
	if [[ ! -d node_modules ]]; then
		npm install
	fi
	npm run build

# Bump tandem to VERSION, validate, commit, tag, push main + tag, and create the GitHub Release.
# Usage: just release 0.2.1
release VERSION:
	#!/usr/bin/env bash
	set -euo pipefail
	version="{{VERSION}}"
	case "$version" in
		v*) echo "Pass the bare semver version, e.g. 0.2.1, not v0.2.1" >&2; exit 2 ;;
	esac
	if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		echo "VERSION must be semver like 0.2.1" >&2
		exit 2
	fi
	if [[ "$(git branch --show-current)" != "main" ]]; then
		echo "Release must run from main" >&2
		exit 2
	fi
	if [[ -n "$(git status --porcelain)" ]]; then
		echo "Working tree must be clean before release" >&2
		git status --short
		exit 2
	fi
	tag="tandem-v${version}"
	if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
		echo "Tag ${tag} already exists" >&2
		exit 2
	fi
	python3 - "$version" <<-'PY'
	import pathlib, re, sys
	version = sys.argv[1]
	cargo = pathlib.Path("tandem/Cargo.toml")
	text = cargo.read_text()
	text, count = re.subn(r'(?m)^version = "[^"]+"$', f'version = "{version}"', text, count=1)
	if count != 1:
	    raise SystemExit("failed to update tandem/Cargo.toml version")
	cargo.write_text(text)

	release = pathlib.Path("tandem/RELEASE.md")
	text = release.read_text()
	text, count = re.subn(r'## v[0-9]+\.[0-9]+\.[0-9]+ \(recommended tag: `tandem-v[0-9]+\.[0-9]+\.[0-9]+`\)', f'## v{version} (recommended tag: `tandem-v{version}`)', text, count=1)
	if count != 1:
	    raise SystemExit("failed to update tandem/RELEASE.md heading")
	text = re.sub(r'tandem-v[0-9]+\.[0-9]+\.[0-9]+', f'tandem-v{version}', text)
	release.write_text(text)
	PY
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
	git add tandem/Cargo.toml tandem/Cargo.lock tandem/RELEASE.md
	git commit -m "Release tandem v${version}"
	git tag -a "$tag" -m "Release tandem v${version}"
	git push origin main
	git push origin "$tag"
	gh release create "$tag" --repo Algorant/tandem --title "Tandem v${version}" --notes-file tandem/RELEASE.md
