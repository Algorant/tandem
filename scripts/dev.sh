#!/usr/bin/env bash
# One native development path: release builds, isolated Git sandboxes, and previews.
set -euo pipefail
repo_root="$(git -C "$(dirname "${BASH_SOURCE[0]}")/.." rev-parse --show-toplevel)"
manifest="$repo_root/tandem/Cargo.toml"
mode="${1:-tui}"
case "$mode" in tui|project|shell|sandbox|build|check) ;; *) echo "Unknown dev mode: $mode" >&2; exit 2;; esac

# Only the default TUI uses the explicitly configured delegated preview slot.
# Invalid/stale routes fail; they never fall back to a different checkout.
workspace=""
seeded=false
if [[ "$mode" == shell && ( ! -t 0 || ! -t 1 ) ]]; then
    echo 'Run just dev-test in a terminal; use just dev-check for automated validation.' >&2
    exit 2
fi
if [[ "$mode" == tui ]]; then
    route="$(git -C "$repo_root" rev-parse --path-format=absolute --git-common-dir)/tandem-dev-preview"
    if [[ -f "$route" ]]; then
        mapfile -t selection < "$route"
        if [[ ${#selection[@]} != 2 ]]; then
            echo "Invalid dev preview route: $route" >&2
            exit 2
        fi
        manifest="${selection[0]}"
        workspace="${selection[1]}"
        test -f "$manifest"
        test -f "$workspace/.tandem/tandem.md"
    fi
fi

build() {
    local package_dir
    package_dir="$(dirname "$(realpath "$manifest")")"
    echo "Building release: $manifest" >&2
    cargo build --manifest-path "$manifest" --target-dir "$package_dir/target" --release
    binary="$package_dir/target/release/tandem"
    test -x "$binary"
}

sandbox() {
    local dir
    dir="$(mktemp -d "${TMPDIR:-/tmp}/tandem-dev.XXXXXX")"
    echo "Creating disposable Git workspace: $dir" >&2
    (
        cd "$dir"
        git init --quiet
        git config user.name 'Tandem dev sandbox'
        git config user.email dev-sandbox@example.invalid
        "$binary" init --title 'Tandem dev sandbox' >/dev/null
        "$binary" add task 'Explore assignment workflows' --kind epic --acceptance 'Inspect the grouped assignments' >/dev/null
        "$binary" add task 'Assignment with a milestone' --parent task-1 --acceptance 'Inspect complete scope and milestone progress' --constraint 'Sandbox only' --validation 'Native assignment read' >/dev/null
        "$binary" add task 'Dependent assignment' --parent task-1 --blocker task-2 --acceptance 'Inspect the blocker context' >/dev/null
        "$binary" add task 'Milestone: try a batched action' --parent task-2 --acceptance 'Progress persists without its own checkpoint' >/dev/null
        "$binary" add task 'Example papercut' --priority low --tag papercut --acceptance 'Inspect filtered papercuts' >/dev/null
        "$binary" add task 'Try Actions: claim, block, resume, deliver, complete' --acceptance 'Use a to exercise the native workflow' --acceptance 'Preserve meaningful evidence in Logs' >/dev/null
        "$binary" add task 'Completed example' --acceptance 'Read final delivery evidence in Logs' >/dev/null
        "$binary" add decision 'Use a disposable Git sandbox' --body 'Dev lifecycle actions must not commit the real project records.' >/dev/null
        "$binary" rules add prefer 'Keep experiments inside this sandbox' >/dev/null
        git add -- .tandem
        git commit --quiet -m 'dev sandbox baseline'
        "$binary" accord claim task-6 --assignee dev-example >/dev/null
        "$binary" accord deliver task-6 --summary 'Example outcome' --evidence 'Native delivery and archive from the dev build' >/dev/null
        "$binary" complete task-6 >/dev/null
        # Lifecycle writes persist without Git; one explicit forward flush makes
        # the sandbox clean for inspection.
        "$binary" checkpoint >/dev/null
    )
    printf '%s\n' "$dir"
}

smoke() {
    command -v jq >/dev/null
    (
        cd "$workspace"
        local before token milestone invalid
        before="$(git rev-parse HEAD)"
        token="$("$binary" assignment task-2 --json | jq -er '.data.definitionToken')"
        milestone="$("$binary" accord claim task-2-1 --assignee dev-smoke --json)"
        jq -e '.ok and .data.recordWritten and .data.checkpoint.status == "batched"' <<< "$milestone" >/dev/null
        [[ "$(git rev-parse HEAD)" == "$before" ]]
        [[ "$("$binary" assignment task-2 --json | jq -er '.data.definitionToken')" == "$token" ]]
        if invalid="$("$binary" accord deliver task-5 --summary 'No evidence must fail' --json)"; then
            echo 'Smoke failure: delivery without evidence succeeded.' >&2
            exit 1
        fi
        jq -e '.ok == false' <<< "$invalid" >/dev/null
        "$binary" show task-5 --json | jq -e '.data.accordStatus == "ready"' >/dev/null
        "$binary" accord claim task-2 --assignee dev-smoke --json | jq -e '.ok and .data.recordWritten and .data.checkpoint.status == "batched"' >/dev/null
        [[ "$(git rev-parse HEAD)" == "$before" ]]
        "$binary" checkpoint --json | jq -e '.ok and .data.checkpoint.status == "checkpointed"' >/dev/null
        [[ "$(git rev-parse HEAD)" != "$before" ]]
        [[ -z "$(git status --porcelain -- .tandem)" ]]
        [[ "$("$binary" assignment task-2 --json | jq -er '.data.definitionToken')" == "$token" ]]
        [[ -z "$(git ls-files -- .tandem/actor-id)" ]]
    )
    echo 'PASS: assignment freshness, evidence rejection, milestone batching, and explicit forward checkpoint flush.'
}

if [[ "$mode" == check ]]; then
    echo 'Running the full native/CLI test suite...' >&2
    cargo test --manifest-path "$manifest"
    "$repo_root/scripts/tests/test_dev.sh"
fi
build
case "$mode" in
    build)
        echo "Built: $binary"
        "$binary" --version
        exit 0
        ;;
    project)
        workspace="$repo_root"
        echo 'Real project mode: lifecycle actions can commit this checkout’s .tandem records.' >&2
        ;;
    *)
        if [[ -z "$workspace" ]]; then
            workspace="$(sandbox)"
            seeded=true
        fi
        ;;
esac
if [[ "$mode" == sandbox ]]; then
    # Keep stdout machine-readable: callers can use cd "$(just dev-sandbox)".
    printf '%s\n' "$workspace"
    exit 0
fi
printf 'Tandem code:      %s\nTandem workspace: %s\n' "$manifest" "$workspace" >&2
if [[ "$mode" == check ]]; then
    smoke
    echo "Sandbox retained for inspection: $workspace"
    exit 0
fi
if $seeded; then
    echo 'Sandbox: task-5 is a standalone workflow; task-2 has milestone task-2-1; task-6 is in Logs.' >&2
    echo 'Temporary workspaces are retained for inspection, never reused or silently deleted.' >&2
fi
cd "$workspace"
if [[ "$mode" == shell ]]; then
    echo 'Dev binary is first on PATH. Try: tandem assignment task-2 --json; tandem tui; git log --oneline' >&2
    PATH="$(dirname "$binary"):$PATH" exec "${SHELL:-bash}"
fi
exec "$binary" tui
