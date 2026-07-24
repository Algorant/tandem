# Tandem development shortcuts

set positional-arguments

# Run the development TUI. A delegated visual task may temporarily route this
# command through Git-local state to its worktree code and a safe fixture.
dev:
	#!/usr/bin/env bash
	set -euo pipefail
	repo_root="$(git rev-parse --show-toplevel)"
	git_common_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
	route_file="$git_common_dir/tandem-dev-preview"
	manifest="$repo_root/tandem/Cargo.toml"
	workspace="$repo_root"
	if [[ -f "$route_file" ]]; then
		mapfile -t route < "$route_file"
		if [[ "${#route[@]}" -ne 2 ]]; then
			echo "Invalid dev preview route: $route_file" >&2
			exit 2
		fi
		manifest="${route[0]}"
		workspace="${route[1]}"
	fi
	test -f "$manifest"
	test -f "$workspace/.tandem/tandem.md"
	echo "Tandem code:      $manifest"
	echo "Tandem workspace: $workspace"
	cd "$workspace"
	exec cargo run --manifest-path "$manifest" -- tui

# Agent/orchestrator helper: configure the one-command delegated preview slot.
[private]
dev-route manifest workspace:
	#!/usr/bin/env bash
	set -euo pipefail
	manifest="$(realpath "{{manifest}}")"
	workspace="$(realpath "{{workspace}}")"
	test -f "$manifest"
	test -f "$workspace/.tandem/tandem.md"
	git_common_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
	printf '%s\n%s\n' "$manifest" "$workspace" > "$git_common_dir/tandem-dev-preview"

# Agent/orchestrator helper: restore `just dev` to the normal checkout.
[private]
dev-reset:
	#!/usr/bin/env bash
	set -euo pipefail
	git_common_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
	rm -f "$git_common_dir/tandem-dev-preview"

# Verify the local docs runtime satisfies Astro's Node floor.
_check-docs-node:
	#!/usr/bin/env bash
	set -euo pipefail
	node -e '
		const [major, minor] = process.versions.node.split(".").map(Number);
		if (major < 22 || (major === 22 && minor < 12)) {
			console.error(`Docs site expects Node >=22.12.0 for Astro; found ${process.version}.`);
			process.exit(1);
		}
	'

# Start the local documentation site with Astro Starlight.
# The Bun dev script syncs ../docs/ into Starlight before serving.
site: _check-docs-node
	#!/usr/bin/env bash
	set -euo pipefail
	cd site
	if [[ ! -d node_modules ]]; then
		bun install
	fi
	bun run dev

# Alias for the local documentation site.
alias docs := site

# Build the static documentation site output in site/dist/.
# Mirrors the GitHub Pages workflow by installing from bun.lock.
site-build: _check-docs-node
	#!/usr/bin/env bash
	set -euo pipefail
	cd site
	bun install --frozen-lockfile
	bun run build

# Validate a prepared VERSION, tag it, publish it, and verify the GitHub Release
# and AUR update.
# Usage: just release 0.6.5
# Before running, add one meaningful ## VERSION section to RELEASES.md.
release VERSION:
	#!/usr/bin/env bash
	set -euo pipefail
	version="{{VERSION}}"
	case "$version" in
		v*) echo "Pass the bare semver version, e.g. 0.6.5, not v0.6.5" >&2; exit 2 ;;
	esac
	if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		echo "VERSION must be semver like 0.6.5" >&2
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

	notes_file="$(mktemp)"
	manifest_file="$(mktemp)"
	release_file="$(mktemp)"
	trap 'rm -f "$notes_file" "$manifest_file" "$release_file"' EXIT
	python3 scripts/release_checks.py notes "$version" "$notes_file"
	python3 scripts/release_checks.py cargo "$version"
	cargo dist manifest --tag "$tag" --artifacts=global --output-format=json --allow-dirty > "$manifest_file"
	python3 scripts/release_checks.py manifest "$notes_file" "$manifest_file"

	cd tandem
	cargo fmt --check
	cargo test
	cargo build --release
	cargo run -- --version
	cargo run -- version
	cd ../site
	bun install --frozen-lockfile
	bun run build
	bun audit --audit-level=high
	cd ..
	bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts extensions/pi-tandem/tests/relationship-smoke.ts
	TANDEM_BIN="$PWD/tandem/target/release/tandem" bun extensions/pi-tandem/tests/smoke.ts
	TANDEM_BIN="$PWD/tandem/target/release/tandem" bun extensions/pi-tandem/tests/relationship-smoke.ts
	TANDEM_BIN="$PWD/tandem/target/release/tandem" bun extensions/pi-tandem/tests/pi-runtime-smoke.ts
	git diff --check
	git tag -a "$tag" -m "Release tandem v${version}"
	git push origin main
	git push origin "$tag"

	repo="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
	wait_for_workflow() {
		local workflow="$1"
		local run_id=""
		local runs=""
		for _ in {1..180}; do
			runs="$(gh run list --repo "$repo" --workflow "$workflow" --limit 100 --json databaseId,headBranch,createdAt)"
			run_id="$(jq -r --arg tag "$tag" '[.[] | select(.headBranch == $tag)] | sort_by(.createdAt) | last | .databaseId // empty' <<<"$runs")"
			if [[ -n "$run_id" ]]; then
				echo "Waiting for $workflow run $run_id for $tag"
				gh run watch "$run_id" --repo "$repo" --exit-status
				return
			fi
			sleep 10
		done
		echo "Timed out waiting for $workflow to start for $tag" >&2
		exit 1
	}
	wait_for_workflow "Release"

	gh release view "$tag" --repo "$repo" --json isDraft,isPrerelease,body,assets > "$release_file"
	python3 scripts/release_checks.py published "$notes_file" "$release_file"
	wait_for_workflow "Update tandem-bin AUR package"
	echo "Release $tag, GitHub assets/notes, and the tandem-bin AUR workflow verified."
