#!/usr/bin/env bash
# Focused release-note and publication assertions for `just release`.
set -euo pipefail

EXPECTED_ASSETS=(
  tandem-installer.sh
  tandem-x86_64-unknown-linux-gnu.tar.xz
  tandem-aarch64-unknown-linux-gnu.tar.xz
  tandem-x86_64-apple-darwin.tar.xz
  tandem-aarch64-apple-darwin.tar.xz
  sha256.sum
)

usage() {
  echo "usage: release_checks.sh {notes VERSION OUTPUT|cargo VERSION|manifest NOTES MANIFEST|published NOTES RELEASE|select-run ROLE RUNS TAG COMMIT NOT_BEFORE}" >&2
  exit 1
}

fail() { echo "$1" >&2; exit 1; }

notes() {
  local version="$1" output="$2" tmpdir count meaningful
  [[ -f RELEASES.md ]] || fail "RELEASES.md is required before a release"
  tmpdir="$(mktemp -d)"
  trap "rm -rf '$tmpdir'" EXIT
  awk -v version="$version" '
    function heading_value(line, value) {
      value = line
      sub(/^##[ \t]+/, "", value)
      sub(/[ \t]+$/, "", value)
      return value
    }
    $0 ~ /^##[ \t]+/ && heading_value($0) == version { count++; active=1; next }
    active && $0 ~ /^##([ \t]|$)/ { active=0 }
    active { print }
    END { print count + 0 > "'"$tmpdir"'/count" }
  ' RELEASES.md >"$tmpdir/body"
  count="$(<"$tmpdir/count")"
  [[ "$count" == 1 ]] || fail "RELEASES.md must contain exactly one '## $version' section; found $count"
  awk '
    { if (!started && $0 ~ /^[[:space:]]*$/) next; started=1; lines[++n]=$0 }
    END {
      while (n > 0 && lines[n] ~ /^[[:space:]]*$/) n--
      if (n > 0) {
        sub(/^[[:space:]]+/, "", lines[1]); sub(/[[:space:]]+$/, "", lines[n])
        for (i=1; i<=n; i++) print lines[i]
      }
    }
  ' "$tmpdir/body" >"$tmpdir/trimmed"
  meaningful="$(grep -vE '^[[:space:]]*$|^[[:space:]]*#{1,6}([[:space:]]|$)' "$tmpdir/trimmed" | grep -E '[[:alnum:]]' || true)"
  [[ -n "$meaningful" ]] || fail "RELEASES.md section $version must contain meaningful release notes"
  cp "$tmpdir/trimmed" "$output"
}

cargo_version() {
  local version="$1" actual
  actual="$(grep -m1 -E '^version = "[^\"]+"$' tandem/Cargo.toml | sed -E 's/^version = "([^"]+)"$/\1/')" || true
  [[ "$actual" == "$version" ]] || fail "tandem/Cargo.toml version must agree with requested version $version"
}

manifest() {
  local notes_path="$1" manifest_path="$2" notes
  notes="$(<"$notes_path")"
  jq -e --arg notes "$notes" '(.announcement_github_body // "") | contains($notes)' "$manifest_path" >/dev/null \
    || fail "cargo-dist announcement body does not include the curated RELEASES.md section"
}

published() {
  local notes_path="$1" release_path="$2" notes missing empty
  notes="$(<"$notes_path")"
  if jq -e '.isDraft == true' "$release_path" >/dev/null; then
    fail "GitHub Release is still a draft"
  fi
  if jq -e '.isPrerelease == true' "$release_path" >/dev/null; then
    fail "GitHub Release is unexpectedly marked prerelease"
  fi
  jq -e --arg notes "$notes" '((.body // "") | contains($notes))' "$release_path" >/dev/null \
    || fail "GitHub Release body does not include the curated RELEASES.md section"
  missing="$(jq -r --argjson expected "$(printf '%s\n' "${EXPECTED_ASSETS[@]}" | jq -R . | jq -s .)" '[.assets[]?.name] as $names | $expected - $names | sort | join(", ")' "$release_path")"
  [[ -z "$missing" ]] || fail "GitHub Release is missing expected assets: $missing"
  empty="$(jq -r --argjson expected "$(printf '%s\n' "${EXPECTED_ASSETS[@]}" | jq -R . | jq -s .)" '[.assets[]? | select((.size // 0) <= 0) | .name] as $empty | [$expected[] | select(. as $name | $empty | index($name))] | sort | join(", ")' "$release_path")"
  [[ -z "$empty" ]] || fail "GitHub Release has empty expected assets: $empty"
}

select_run() {
  local role="$1" runs="$2" tag="$3" commit="$4" not_before="$5"
  case "$role" in release|aur) ;; *) fail "unknown workflow role: $role" ;; esac
  jq -c --arg role "$role" --arg tag "$tag" --arg commit "$commit" --arg not_before "$not_before" '
    [.[] | select(
      if $role == "release" then .event == "push" and .headBranch == $tag and .headSha == $commit
      else .event == "workflow_run" and .headSha == $commit and (.createdAt // "") >= $not_before
      end
    )] | if length == 0 then empty else reduce .[] as $run (null; if . == null or (($run.createdAt // "") > (.createdAt // "")) then $run else . end) end
  ' "$runs"
}

[[ $# -ge 1 ]] || usage
command="$1"; shift
case "$command" in
  notes) [[ $# -eq 2 ]] || usage; notes "$@" ;;
  cargo) [[ $# -eq 1 ]] || usage; cargo_version "$@" ;;
  manifest) [[ $# -eq 2 ]] || usage; manifest "$@" ;;
  published) [[ $# -eq 2 ]] || usage; published "$@" ;;
  select-run) [[ $# -eq 5 ]] || usage; select_run "$@" ;;
  *) usage ;;
esac
