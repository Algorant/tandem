# Tandem development shortcuts

set positional-arguments

# Run the development TUI against this repository's .tandem workspace.
dev:
	cargo run --manifest-path tandem/Cargo.toml -- tui

# Verify the local docs runtime matches site/.node-version and Astro's Node floor.
_check-docs-node:
	#!/usr/bin/env bash
	set -euo pipefail
	expected="$(tr -d '[:space:]' < site/.node-version)"
	node -e '
		const expected = process.argv[1];
		const [major, minor] = process.versions.node.split(".").map(Number);
		if (String(major) !== expected || (major === 22 && minor < 12)) {
			console.error(`Docs site expects Node ${expected}.x from site/.node-version (Astro requires >=22.12.0); found ${process.version}.`);
			process.exit(1);
		}
	' "$expected"

# Start the local documentation site with Astro Starlight.
# The npm dev script syncs ../docs/ into Starlight before serving.
site: _check-docs-node
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
# Mirrors the GitHub Pages workflow by installing from package-lock.json.
site-build: _check-docs-node
	#!/usr/bin/env bash
	set -euo pipefail
	cd site
	npm ci
	npm run build

# Bump tandem to VERSION, validate, commit, tag, push main + tag, and create a concise GitHub Release.
# Usage: just release 0.2.1
# Before running, curate tandem/GITHUB_RELEASE_NOTES.md; tandem/RELEASE.md is the reusable checklist.
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

	notes = pathlib.Path("tandem/GITHUB_RELEASE_NOTES.md")
	text = notes.read_text()
	text, count = re.subn(r'(?m)^# Tandem v[0-9]+\.[0-9]+\.[0-9]+$', f'# Tandem v{version}', text, count=1)
	if count != 1:
	    raise SystemExit("failed to update tandem/GITHUB_RELEASE_NOTES.md heading")
	text = re.sub(r'tandem-v[0-9]+\.[0-9]+\.[0-9]+', f'tandem-v{version}', text)
	notes.write_text(text)
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
	git add tandem/Cargo.toml tandem/Cargo.lock tandem/RELEASE.md tandem/GITHUB_RELEASE_NOTES.md
	git commit -m "Release tandem v${version}"
	git tag -a "$tag" -m "Release tandem v${version}"
	git push origin main
	git push origin "$tag"
	gh release create "$tag" --repo Algorant/tandem --title "Tandem v${version}" --notes-file tandem/GITHUB_RELEASE_NOTES.md
