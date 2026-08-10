---
id: task-223
type: task
title: "Prepare Tandem v0.10.0 Papercuts release"
state: "validation"
priority: "high"
references: ["task-222", "decision-5"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "README.md", "tandem/RELEASE.md", "justfile", "scripts/release_checks.py", "site/package.json", "site/bun.lock"]
tags: ["chore", "release", "papercuts"]
createdAt: "2026-08-10T12:46:07Z"
updatedAt: "2026-08-10T15:03:29Z"
accord:
  status: "blocked"
  assignee: "pi"
  claimedAt: "2026-08-10T12:46:12Z"
  deliveredAt: "2026-08-10T13:00:00Z"
  validation:
    commands: ["cargo fmt --check passed", "cargo test passed: 246 unit and 11 integration tests", "cargo build --release and cargo build --profile dist passed", "cargo clippy --all-targets --all-features -- -D warnings passed", "Release and debug binaries report tandem 0.10.0", "Release binary Papercut add/list/show/search/resolve dogfood passed in a temporary workspace", "pi-tandem static check, smoke, relationship smoke, and project-local Pi runtime smoke passed against the release binary", "Docs build produced 19 pages and link check passed 912 internal links", "Bun audit passed after updating docs dependencies and overriding patched nanoid/js-yaml versions", "Web JavaScript syntax checks passed", "release_checks.py tests and 0.10.0 notes/version checks passed", "cargo-dist 0.32.0 manifest includes the curated 0.10.0 notes", "just release 0.10.0 dry run uses the installed dist binary and contains the expected validation/publication workflow", "GitHub auth and repository target verified; tandem-v0.10.0 does not exist locally or remotely", "AUR is currently reachable; tandem-bin remains at 0.8.1-1 after prior maintenance-related workflow failures"]
  summary: "Prepared and fully validated the Tandem v0.10.0 Papercuts release candidate without tagging, pushing, or publishing."
  evidence: ["Release preparation commit 6a25eb3", "Secure docs dependency refresh commit e589d7c", "Release command fix commit a19ec9a", "Working tree was clean before delivery", "No tag, push, GitHub Release, AUR update, or published install occurred"]
  filesChanged: ["README.md", "RELEASES.md", "tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/RELEASE.md", "site/package.json", "site/bun.lock", "justfile"]
  reason: "GitHub Release v0.10.0 and all 13 assets are published and verified, but the required tandem-bin AUR update cannot clone ssh://aur@aur.archlinux.org/tandem-bin.git because AUR SSH reports scheduled maintenance. Automatic workflow 31401300611 and manual retry 31401441667 both failed at the same upstream maintenance response. Keep the release Task open until the AUR workflow succeeds."
  updatedAt: "2026-08-10T15:03:29Z"
assignee: "pi"
---

## Description

## Outcome

Prepare a clean, fully validated Tandem v0.10.0 release candidate for the Papercuts inbox MVP.

## Scope

- Update `tandem/Cargo.toml` and `tandem/Cargo.lock` to `0.10.0`.
- Add concise curated `0.10.0` public notes to `RELEASES.md` following decision-5.
- Update current source-install examples and release capability documentation.
- Build and manually dogfood the release binary.
- Run the complete Rust, cargo-dist, pi-tandem, documentation, JavaScript, Bun audit, and diff validation suite.
- Verify the repository is clean and ready for `just release 0.10.0`.

## Publication boundary

This Task begins release preparation and validation. Do not create or push the tag, push `main`, publish the GitHub Release, update AUR, or install the published release until the product owner reviews the release candidate and explicitly approves publication.

## Proposed public positioning

Tandem v0.10.0 introduces Papercuts: a lightweight project inbox for preserving small, non-blocking friction without interrupting active work. It includes CLI commands, global search, loose Task and Decision references, the thin `pi-tandem` tool, and complete documentation. Existing workspaces require no migration.

## Acceptance criteria

1. Package and lockfile versions are `0.10.0`.
2. `RELEASES.md` contains exactly one meaningful `## 0.10.0` section with no installation guidance.
3. Release notes use Features, Improvements, and Compatibility sections and describe only shipped behavior.
4. Release documentation lists the Papercut CLI and optional `.tandem/papercuts/` storage.
5. The release build completes add/list/show/search/resolve Papercut dogfood in a temporary workspace.
6. All commands required by `tandem/RELEASE.md` and `just release` pass without tagging or publishing.
7. cargo-dist includes the curated notes and expected release plan.
8. The final checkout is clean and the exact publication command is presented for approval.
