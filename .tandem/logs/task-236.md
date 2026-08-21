---
id: task-236
type: task
title: "Release Tandem 0.10.3"
priority: "high"
effort: "medium"
references: ["task-235", "task-231"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "justfile"]
tags: ["chore", "release", "web"]
createdAt: "2026-08-21T21:35:03Z"
updatedAt: "2026-08-21T21:41:45Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-08-21T21:35:09Z"
  deliveredAt: "2026-08-21T21:41:36Z"
  deliverables: ["Version 0.10.3 metadata and curated release notes", "Pushed main and annotated tandem-v0.10.3 tag", "Published GitHub Release with 13 expected assets", "Verified branded primary installer installs tandem 0.10.3"]
  validation:
    commands: ["`just release 0.10.3` completed successfully", "Release workflow 32529378457 succeeded", "GitHub Release is published, non-draft, and non-prerelease", "All four platform archives, checksums, source, manifest, and installer are present", "`curl -fsSL https://trytandem.dev/install.sh | sh` installed tandem 0.10.3 to /home/ivan/.cargo/bin", "Branded installer URL returns HTTP 302 to the latest GitHub Release installer"]
  summary: "Published and verified Tandem v0.10.3 with the web reference-validation performance fix."
  evidence: ["Release commit f4be0c252d7a52ff71588d50e6020627fdbda1cd", "Annotated tag tandem-v0.10.3 points to release commit", "GitHub Release https://github.com/Algorant/tandem/releases/tag/tandem-v0.10.3", "AUR verification intentionally skipped because AUR is read-only, per repository release policy"]
  filesChanged: ["RELEASES.md", "tandem/Cargo.toml", "tandem/Cargo.lock", ".tandem/board/task-236.md", ".tandem/events/3ba2bee6-e75b-41ec-8124-f68506643fea.jsonl"]
  reviewer: "pi-orchestrator"
  note: "Accepted after full release validation, successful GitHub Actions publication, asset verification, and primary installer smoke test."
  updatedAt: "2026-08-21T21:41:40Z"
assignee: "pi"
completedAt: "2026-08-21T21:41:45Z"
completion:
  summary: "Released Tandem v0.10.3. Pushed main and the annotated tag, published and verified the GitHub Release with 13 assets, and confirmed the branded primary installer installs tandem 0.10.3. AUR verification remained intentionally skipped under the current read-only AUR policy."
  filesChanged: ["RELEASES.md", "tandem/Cargo.toml", "tandem/Cargo.lock", ".tandem/logs/task-236.md", ".tandem/events/3ba2bee6-e75b-41ec-8124-f68506643fea.jsonl"]
  validation: "`just release 0.10.3` passed; Release workflow 32529378457 succeeded; GitHub Release and 13 assets verified; branded installer redirected correctly and installed tandem 0.10.3 to /home/ivan/.cargo/bin."
  reviewer: "pi-orchestrator"
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
