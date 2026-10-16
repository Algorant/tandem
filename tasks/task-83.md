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
updatedAt: "2026-10-16T12:24:58Z"
assignee: "tandem"
---

## Description

Algorant via Herdra approved the order: accepted visuals → write and review seven-page docs → after Algorant docs acceptance, release 0.17.1 with the fix batch and docs through the new release gates. Task-81 is integrated/completed; code fixes are local and unpushed. Task-76 is plan-first and gated on Herdra plan/rendered review, then Algorant acceptance. Old-URL handling must be brought with the rendered pages and decided before publication.

Do not start this release while task-76 remains unaccepted. Follow maintained tandem/RELEASE.md, including annotated tag + GitHub Release object, configured asset/checksum verification, isolated installer, pinned cargo-dist and bounded nonblocking AUR verification. Preserve the already-published 0.17.0 release; no force tags or new network infrastructure.
