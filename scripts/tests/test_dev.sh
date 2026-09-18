#!/usr/bin/env bash
# Exercise the actual maintained Just recipe and native CLI in isolated Git repos.
set -euo pipefail
repo_root="$(git -C "$(dirname "${BASH_SOURCE[0]}")/../.." rev-parse --show-toplevel)"
cd "$repo_root"
head_before="$(git rev-parse HEAD)"
index_before="$(git ls-files --stage)"
records_before="$(git diff HEAD -- .tandem)"
first=""
second=""
cleanup() {
    # These exact directories were allocated by this test's native recipe calls.
    [[ -z "$first" ]] || rm -rf -- "$first"
    [[ -z "$second" ]] || rm -rf -- "$second"
}
trap cleanup EXIT
first="$(just dev-sandbox)"
second="$(just dev-sandbox)"
[[ "$first" != "$second" ]]
binary="$repo_root/tandem/target/release/tandem"
for sandbox in "$first" "$second"; do
    [[ -d "$sandbox/.git" && "$sandbox" != "$repo_root" ]]
    [[ "$(git -C "$sandbox" rev-parse --show-toplevel)" == "$sandbox" ]]
    [[ "$(git -C "$sandbox" rev-list --count HEAD)" == 2 ]]
    [[ -z "$(git -C "$sandbox" status --porcelain)" ]]
    [[ -z "$(git -C "$sandbox" ls-files -- .tandem/actor-id)" ]]
    (
        cd "$sandbox"
        "$binary" show task-5 --json | jq -e '.data.accordStatus == "ready" and .data.location == "board"' >/dev/null
        "$binary" show task-6 --json | jq -e '.data.location == "logs" and (.data.accord.evidence | length > 0)' >/dev/null
        "$binary" assignment task-2 --json | jq -e '.data.root.role == "task" and .data.milestones[0].id == "task-2-1"' >/dev/null
    )
done
[[ "$(git rev-parse HEAD)" == "$head_before" ]]
[[ "$(git ls-files --stage)" == "$index_before" ]]
[[ "$(git diff HEAD -- .tandem)" == "$records_before" ]]
[[ "$(just --dry-run dev 2>&1)" == *'./scripts/dev.sh tui'* ]]
[[ "$(just --dry-run dev-project 2>&1)" == *'./scripts/dev.sh project'* ]]
echo 'PASS: Just routes dev safely; independent native sandboxes preserve project HEAD, index and records.'
