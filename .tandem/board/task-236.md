---
id: task-236
type: task
title: "Release Tandem 0.10.3"
state: "in-progress"
priority: "high"
effort: "medium"
references: ["task-235", "task-231"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "justfile"]
tags: ["chore", "release", "web"]
createdAt: "2026-08-21T21:35:03Z"
updatedAt: "2026-08-21T21:35:09Z"
accord:
  status: "claimed"
  assignee: "pi"
  claimedAt: "2026-08-21T21:35:09Z"
  updatedAt: "2026-08-21T21:35:09Z"
assignee: "pi"
---

## Description

Prepare, publish, and verify Tandem v0.10.3 as a patch release containing the Tandem web reference-validation performance fix from task-235.

Scope:
- Set the Rust package and lockfile version to 0.10.3.
- Add concise curated 0.10.3 notes to RELEASES.md.
- Run the complete repository release validation through `just release 0.10.3`.
- Push `main` and annotated tag `tandem-v0.10.3`.
- Verify the GitHub Release, notes, assets, checksums, and primary installer.
- Record downstream AUR outcome according to current project policy.

Acceptance criteria:
1. Cargo.toml and Cargo.lock report 0.10.3 for Tandem.
2. RELEASES.md contains one meaningful 0.10.3 section.
3. Full release checks pass.
4. Main and the annotated release tag are pushed.
5. The non-draft, non-prerelease GitHub Release is published with expected assets and curated notes.
6. The primary installer reports tandem 0.10.3.
7. Release and downstream packaging outcomes are recorded in Tandem.
