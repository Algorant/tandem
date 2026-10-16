---
id: task-83
uid: 2d604094-079a-4f7e-962f-e9c26925d309
type: task
title: "Release Tandem 0.17.1 after accepted docs"
state: "in-progress"
priority: "high"
effort: "medium"
blockers: ["task-76"]
references: ["task-81", "task-27", "task-50", "task-64", "task-63", "task-77"]
relatedFiles: ["tandem/RELEASE.md", "scripts/release_checks.sh", "dist-workspace.toml", "tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", ".github/workflows/release.yml", ".github/workflows/aur-tandem-bin.yml"]
tags: ["release"]
accord:
  status: "claimed"
  acceptance: ["Only after Algorant accepts the rendered task-76 docs and old-URL handling: release 0.17.1 includes the integrated fix batch, accepted visuals and docs, with version and curated release notes correct.", "Maintained release gates pass, including native dev-check with strict clippy, site build/link/high-severity audit gates and pinned cargo-dist; no global tooling changes.", "Annotated tandem-v0.17.1 tag and GitHub Release are published with every configured release asset and valid checksums; a clean isolated installer installs the released 0.17.1 binary.", "AUR publication and bounded metadata verification are reported; a downstream AUR failure does not block an otherwise verified GitHub release. No unapproved network infrastructure or publication before docs acceptance."]
  claimedAt: "2026-10-16T12:24:58Z"
  validation: ["$ just dev-check", "$ just site-build", "$ cd site && bun run check:links"]
  constraints: ["Algorant explicitly approved release 0.17.1 via Herdra on 2026-10-14, ordered AFTER docs acceptance. This record captures authorization, not permission to start before task-76 completes.", "Plan first, Luna/high if delegated. Follow current tandem/RELEASE.md and new release gates; do not reintroduce removed release machinery.", "No source push, tag push or docs deployment until Algorant accepts docs/URL handling. Release publication is then authorized; surface any separate deployment decision that remains unresolved.", "Temporary verification installs/downloads only; do not commit runtime state or add permanent tests/fixtures."]
  updatedAt: "2026-10-16T12:24:58Z"
createdAt: "2026-10-14T04:01:41Z"
updatedAt: "2026-10-16T12:24:59Z"
assignee: "tandem"
---
## Explicit publication authorization

Algorant via Herdra approved docs rev5 and release 0.17.1. Parent has finished task-76 through owning Worker lifecycle; main190ad105 includes five held fix/visual commits plus accepted docs. Authorized order: push main, deploy site, verify live pages/redirects; prepare/version/curated notes and run maintained just release0.17.1; verify GitHub assets/checksums, AUR and exact live-site install line. No additional approval gate remains. Stop all browser/preview processes used.

## Direct operator plan

This release is tightly coupled operator publication; parent executes on cleanmain, not a separate publishing Worker. Verify merged site build/link/highaudit before push; match Docs workflow to pushed190ad105 and wait for successful deployment; verify sevenlivecanonicalpages, twelve aliases and three404s plus media/installerURL. Then update Cargo package+lock to0.17.1, add one concise RELEASES section for fixes/docs, commit, and run the authoritative justrelease recipe with pinned cargo-dist/all newgates. Verify all configured published assets/checksums with isolated installer; observe/report bounded nonblocking AUR result. No remote tag rewriting or new network infra. Current docs accepted diagrams/crop unchanged. Retained preview processes already stopped; restart none unless live verification requires Obscura, close it immediately afterward.

Known old README-like prose in maintainer RELEASE.md is not release behavior; do not expand this release into unrelated guide cleanup.