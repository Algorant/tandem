---
id: task-31
type: task
title: "Publish Tandem 0.13.1 reference-links patch release"
effort: "medium"
references: ["task-29", "task-30", "decision-4", "task-1"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "site/bun.lock"]
tags: ["docs", "release"]
accord:
  status: "accepted"
  acceptance: ["Package and lockfile identify 0.13.1, with concise curated notes for the approved reference warning/web/TUI changes.", "Required release checks pass; main and annotated tandem-v0.13.1 tag are pushed, and a non-draft GitHub Release contains the notes, all platform assets, installer, and checksums.", "Primary installer resolves to 0.13.1 and an isolated installation reports tandem 0.13.1; record downstream AUR status."]
  claimedAt: "2026-09-10T18:57:30Z"
  deliveredAt: "2026-09-10T19:07:22Z"
  validation: ["$ just release 0.13.1", "Verify primary installer and installed version in disposable state."]
  constraints: ["Use the established just release path; no release-automation or adapter changes.", "Resolve the observed GHSA-7w5x-hrqm-74c2 docs dependency audit blocker with a minimal in-range lockfile update; do not bypass the security audit."]
  summary: "Published Tandem 0.13.1 after Algorant approved task-29/task-30. Version, curated notes, and minimal docs security dependency patch are committed and pushed; release and AUR workflows succeeded."
  evidence: ["just release 0.13.1 rerun exited 0: Rust release tests/build/dist/clippy, docs build/audit, web syntax, adapter smoke suites, annotated tag push, Release workflow and published asset/notes checks passed. First attempt stopped before tag/push on smol-toml GHSA-7w5x-hrqm-74c2; the sole lockfile package update 1.7.0 -> 1.7.1 resolved it and bun audit reported no vulnerabilities.", "Annotated tandem-v0.13.1 tag points to f0ca7693f86fa8b8a00a51d6dbc5069c7c76c1cc, matching pushed origin/main at publication; GitHub Release https://github.com/Algorant/tandem/releases/tag/tandem-v0.13.1 is published, non-draft and non-prerelease with curated notes and all four platform archives, installer, aggregate and per-archive checksums.", "/tmp/tandem-reference-review.rdSmyL/verify-release.sh exited 0: downloaded all release assets, verified aggregate and all per-archive SHA-256 checksums, confirmed https://trytandem.dev/install.sh matches the published installer byte-for-byte, installed into isolated temporary HOME without PATH/profile modifications, and asserted tandem 0.13.1 plus warning-free reference smoke.", "AUR workflow https://github.com/Algorant/tandem/actions/runs/34518338245 completed successfully including Commit and push AUR update. The old read-only limitation did not apply to this publication; no release automation was changed."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "site/bun.lock"]
  updatedAt: "2026-09-10T19:07:26Z"
createdAt: "2026-09-10T18:57:26Z"
updatedAt: "2026-09-10T19:07:26Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-10T19:07:26Z"
resolution:
  outcome: "completed"
---

## Description

Algorant approved the task-29/task-30 visual preview and explicitly requested committing, pushing, and the next patch release. Current package and latest GitHub Release are 0.13.0. Prepare and publish 0.13.1 using the established just release path; do not change release automation or adapters. Record AUR separately as non-blocking.
