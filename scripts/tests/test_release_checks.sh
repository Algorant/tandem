#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
checks="$repo_root/scripts/release_checks.sh"
fixture="$repo_root/scripts/tests/fixtures/workflow-runs.json"
workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

assert_eq() {
  [[ "$1" == "$2" ]] || { echo "expected '$2', got '$1'" >&2; exit 1; }
}

release="$(jq -c '.release' "$fixture")"
aur="$(jq -c '.aur' "$fixture")"
selected="$(printf '%s' "$release" >"$workdir/release-runs.json"; "$checks" select-run release "$workdir/release-runs.json" tandem-v0.6.5 release-commit '')"
assert_eq "$(jq -r .databaseId <<<"$selected")" 9001
boundary="$(jq -r '.release[0].updatedAt' "$fixture")"
printf '%s' "$aur" >"$workdir/aur-runs.json"
selected="$($checks select-run aur "$workdir/aur-runs.json" tandem-v0.6.5 release-commit "$boundary")"
assert_eq "$(jq -r .databaseId <<<"$selected")" 9104
assert_eq "$(jq -r .headBranch <<<"$selected")" main

cat >"$workdir/tie-runs.json" <<'JSON'
[
  {"databaseId":1,"event":"push","headBranch":"tag","headSha":"commit","createdAt":"2026-01-01T00:00:00Z"},
  {"databaseId":2,"event":"push","headBranch":"tag","headSha":"commit","createdAt":"2026-01-01T00:00:00Z"}
]
JSON
selected="$($checks select-run release "$workdir/tie-runs.json" tag commit '')"
assert_eq "$(jq -r .databaseId <<<"$selected")" 1

no_match="$($checks select-run release "$workdir/release-runs.json" wrong-tag release-commit '' || true)"
assert_eq "$no_match" ""

echo "release_checks.sh workflow selection tests: passed"
