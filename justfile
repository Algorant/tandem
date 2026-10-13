# Tandem development shortcuts

set positional-arguments

# An explicit delegated preview route selects that worktree and fixture instead.
# Build release and open a fresh, disposable Git-backed TUI sandbox.
dev:
	@./scripts/dev.sh tui

# Explicitly run the release TUI on real project records (actions may commit them).
dev-project:
	@./scripts/dev.sh project

# Run the fixture-driven release workflow selector checks.
test-release-checks:
	@scripts/tests/test_release_checks.sh

# Start the local read-only Tandem web interface and open it in the browser.
web:
	#!/usr/bin/env bash
	set -euo pipefail
	repo_root="$(git rev-parse --show-toplevel)"
	cd "$repo_root"
	exec cargo run --manifest-path tandem/Cargo.toml -- web

# Agent/orchestrator helper: configure the one-command delegated preview with a session-only seed.
[private]
dev-route manifest workspace seed_workspace:
	#!/usr/bin/env bash
	set -euo pipefail
	manifest="$(realpath "{{manifest}}")"
	workspace="$(realpath "{{workspace}}")"
	seed_workspace="$(realpath "{{seed_workspace}}")"
	test -f "$manifest"
	test -f "$workspace/.tandem/tandem.md"
	test -f "$seed_workspace/.tandem/tandem.md"
	git_common_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
	printf '%s\n%s\n%s\n' "$manifest" "$workspace" "$seed_workspace" > "$git_common_dir/tandem-dev-preview"

# Agent/orchestrator helper: restore `just dev` to its safe sandbox default.
[private]
dev-reset:
	#!/usr/bin/env bash
	set -euo pipefail
	git_common_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
	rm -f "$git_common_dir/tandem-dev-preview"

# Build the latest dev release binary from this checkout, without installation.
dev-build:
	@./scripts/dev.sh build

# Create a fresh seeded Git sandbox; print its absolute path on stdout.
dev-sandbox:
	@./scripts/dev.sh sandbox

# Open a shell in a fresh Git sandbox with the release binary first on PATH.
dev-test:
	@./scripts/dev.sh shell

# Run native/CLI tests and a Git-backed assignment and board smoke test.
dev-check:
	@./scripts/dev.sh check

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

# Validate a prepared VERSION, tag it, publish it, and verify GitHub plus downstream AUR workflows.
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
	# Resolve cargo-dist from the repository's sole version authority before doing release work.
	if ! dist_version="$(awk -F'"' '$1 ~ /^[[:space:]]*cargo-dist-version = / { print $2; count++ } END { if (count != 1) exit 1 }' dist-workspace.toml)" || [[ ! "$dist_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		echo "Could not read one valid cargo-dist-version from dist-workspace.toml." >&2
		exit 2
	fi
	if ! command -v mise >/dev/null 2>&1; then
		echo "mise is required to provision pinned cargo-dist ${dist_version}; install mise, then retry." >&2
		exit 2
	fi
	dist_cmd=(mise x "aqua:axodotdev/cargo-dist@${dist_version}" -- dist)
	if ! dist_version_output="$("${dist_cmd[@]}" --version)"; then
		echo "Unable to provision/run cargo-dist ${dist_version} through mise; check mise and its aqua backend, then retry." >&2
		exit 2
	fi
	if [[ "$dist_version_output" != "cargo-dist ${dist_version}" ]]; then
		echo "Expected cargo-dist ${dist_version} from dist-workspace.toml, got: ${dist_version_output}" >&2
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
	runs_file="$(mktemp)"
	trap 'rm -f "$notes_file" "$manifest_file" "$release_file" "$runs_file"' EXIT
	scripts/release_checks.sh notes "$version" "$notes_file"
	scripts/release_checks.sh cargo "$version"
	"${dist_cmd[@]}" manifest --tag "$tag" --artifacts=global --output-format=json --allow-dirty > "$manifest_file"
	scripts/release_checks.sh manifest "$notes_file" "$manifest_file"

	cd tandem
	cargo fmt --check
	# Test the same optimized profile used by the release binary so the later
	# release build reuses the test compilation instead of building debug first.
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
	bun build --no-bundle tandem/src/web/app.js tandem/src/web/api.js tandem/src/web/ui.js --outdir /tmp/tandem-web-syntax-check
	git diff --check
	git tag -a "$tag" -m "Release tandem v${version}"
	git push origin main
	git push origin "$tag"

	repo="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
	release_commit="$(git rev-parse "${tag}^{}")"
	release_run_completed_at=""
	wait_for_workflow() {
		local role="$1"
		local workflow="$2"
		local max_attempts="$3"
		local watch_attempts="$4"
		local selection=""
		local run_id=""
		local run_status=""
		for ((attempt = 1; attempt <= max_attempts; attempt++)); do
			gh run list --repo "$repo" --workflow "$workflow" --limit 100 --json databaseId,headBranch,headSha,event,createdAt,updatedAt > "$runs_file" || return $?
			selection="$(scripts/release_checks.sh select-run "$role" "$runs_file" "$tag" "$release_commit" "$release_run_completed_at")"
			if [[ -n "$selection" ]]; then
				run_id="$(jq -r .databaseId <<<"$selection")"
				echo "Waiting for $workflow run $run_id for $tag"
				if [[ -n "$watch_attempts" ]]; then
					for ((watch_attempt = 1; watch_attempt <= watch_attempts; watch_attempt++)); do
						run_status="$(gh run view "$run_id" --repo "$repo" --json status,conclusion)" || return $?
						if [[ "$(jq -r .status <<<"$run_status")" == completed ]]; then
							if [[ "$(jq -r .conclusion <<<"$run_status")" != success ]]; then
								echo "$workflow run $run_id completed with conclusion $(jq -r .conclusion <<<"$run_status")." >&2
								return 1
							fi
							break
						fi
						sleep 10
					done
					if [[ "$(jq -r .status <<<"$run_status")" != completed ]]; then
						echo "Timed out waiting for $workflow run $run_id after $((watch_attempts * 10)) seconds" >&2
						return 124
					fi
				else
					gh run watch "$run_id" --repo "$repo" --exit-status || return $?
				fi
				if [[ "$role" == "release" ]]; then
					release_run_completed_at="$(gh run view "$run_id" --repo "$repo" --json updatedAt --jq .updatedAt)"
				fi
				return 0
			fi
			sleep 10
		done
		echo "Timed out waiting for $workflow to start for $tag after $((max_attempts * 10)) seconds" >&2
		return 124
	}
	wait_for_workflow release "Release" 180 ""

	gh release view "$tag" --repo "$repo" --json isDraft,isPrerelease,body,assets > "$release_file"
	scripts/release_checks.sh published "$notes_file" "$release_file"
	echo "GitHub Release $tag and required assets/notes verified."
	if wait_for_workflow aur "Update tandem-bin AUR package" 30 30; then
		echo "AUR workflow verified for $tag."
	else
		aur_status=$?
		echo "WARNING: matching AUR workflow verification did not succeed (exit $aur_status); GitHub Release $tag remains verified. Record a downstream tandem-bin packaging follow-up." >&2
	fi
	echo "Release $tag and GitHub assets/notes verified."
