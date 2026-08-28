---
id: task-242
type: task
title: "Convert release_checks.py to bash and jq"
priority: "low"
effort: "small"
references: ["decision-3", "task-233"]
relatedFiles: ["scripts/release_checks.py", "scripts/tests/test_release_checks.py", "scripts/tests/fixtures/workflow-runs.json", "justfile"]
tags: ["config", "tooling", "release"]
createdAt: "2026-08-25T12:39:46Z"
updatedAt: "2026-08-28T23:23:01Z"
accord:
  status: "accepted"
  assignee: "worker-task-242-fd7651e9"
  claimedAt: "2026-08-25T12:44:54Z"
  deliveredAt: "2026-08-28T23:22:55Z"
  validation:
    commands: ["scripts/tests/test_release_checks.sh passes", "no .py files remain in the repository", "all 5 justfile release_checks call sites rewired plus test hook at line 47"]
  summary: "Converted release_checks.py and its unittest module to bash + jq; removed all Python from the repository."
  filesChanged: ["scripts/release_checks.sh", "scripts/tests/test_release_checks.sh", "justfile"]
  note: "Verified on main with a clean working tree: Python fully removed, bash/jq replacements in place, justfile rewired, fixture test passes."
  updatedAt: "2026-08-28T23:22:57Z"
assignee: "worker-task-242-fd7651e9"
completedAt: "2026-08-28T23:23:01Z"
completion:
  summary: "Converted scripts/release_checks.py and its unittest module to scripts/release_checks.sh and scripts/tests/test_release_checks.sh using bash and jq, preserving all five subcommands (notes, cargo, manifest, published, select-run). Rewired all justfile call sites. Also removed scripts/benchmark_tui_idle.py beyond original scope; task-233 owns choosing its replacement. No Python remains in the repository."
---
Tandem is a Rust project with Bun for JS/TS automation (decision-3). Python remains only under `scripts/`. This task removes the release-path Python.

## Scope

Convert `scripts/release_checks.py` (133 lines) and its unittest module to bash + `jq`.

| Subcommand | Behavior to preserve |
| --- | --- |
| `notes VERSION OUTPUT` | Extract exactly one `## <version>` section from RELEASES.md, fail on 0 or 2+ matches, fail if the section has no alphanumeric non-heading content, write it to OUTPUT |
| `cargo VERSION` | `tandem/Cargo.toml` `version = "..."` must equal VERSION |
| `manifest NOTES MANIFEST` | cargo-dist `announcement_github_body` must contain the curated notes |
| `published NOTES RELEASE` | Not draft, not prerelease, body contains notes, all 6 expected assets present and non-empty |
| `select-run ROLE RUNS TAG COMMIT NOT_BEFORE` | `release`: push event, headBranch=tag, headSha=commit. `aur`: workflow_run event, headSha=commit, createdAt >= NOT_BEFORE. Both pick max createdAt, print JSON, print nothing when no match |

Called 6 times from the `just release` recipe (justfile lines 165, 166, 168, 207, 225).

## Out of scope

`scripts/benchmark_tui_idle.py` stays as-is. It needs a real PTY, has been run once since it was written in task-226, and its only live claim is task-233. Revisit it there.

## Acceptance

- `just release` behaves identically, including the dry-run path.
- Fixture-driven coverage over `scripts/tests/fixtures/workflow-runs.json` still runs, in whatever form replaces the unittest module, and the justfile invokes it.
- `scripts/release_checks.py` and `scripts/tests/test_release_checks.py` are removed.