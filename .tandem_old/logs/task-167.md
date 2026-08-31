---
id: task-167
type: task
title: "Release Task body editing and cancellation as Tandem v0.6.1"
priority: "high"
blockers: ["task-166"]
references: ["task-144", "task-146", "decision-5"]
relatedFiles: ["tandem/Cargo.toml", "tandem/GITHUB_RELEASE_NOTES.md", "tandem/RELEASE.md", "README.md", "tandem/README.md", ".github/workflows/release.yml", ".github/workflows/aur-tandem-bin.yml"]
tags: ["release", "cli", "tui", "pi-tandem", "protocol"]
createdAt: "2026-07-23T01:21:04Z"
updatedAt: "2026-07-23T02:05:15Z"
accord:
  status: "accepted"
  assignee: "parent-agent"
  claimedAt: "2026-07-23T01:52:15Z"
  deliveredAt: "2026-07-23T02:04:53Z"
  deliverables: ["GitHub Release: https://github.com/Algorant/tandem/releases/tag/tandem-v0.6.1", "cargo-dist run 29973001468 completed successfully for all four target archives and global/host jobs", "AUR run 29973139232 completed successfully; tandem-bin commit bc4c7b3f4dde220d8b85ebe9d829269fc2543bb2 publishes 0.6.1-1", "Published notes contain a dedicated Bug fixes section and no installation guidance"]
  validation:
    commands: ["just release 0.6.1: cargo fmt, 161 Rust tests, release build/version checks, Bun frozen install/site build/audit, pi-tandem syntax and three runtime smokes all passed", "Docs link checker passed: 605 internal links across 15 HTML pages", "Downloaded all 13 GitHub Release assets; aggregate and five per-artifact SHA-256 checks passed", "Extracted x86_64 Linux binary reports 0.6.1 and passed real update --body, cancel, JSON/Log, body-preservation, and event-privacy smoke", "Isolated cargo-dist installer installed 0.6.1 under /tmp only", "AUR PKGBUILD/.SRCINFO version 0.6.1 checksum 98fda50355f9f175de9c3eb633cb69c34790388a9829a9a0b5c3f1481346bc25 matches the published x86_64 archive", "origin/main, local main, and peeled annotated tag all resolve to 3eb9309570ed5b8d746e0f9f7ea5414f20526182; working tree is clean", "Protocol references remain at 0.1.0; /usr/bin/tandem pacman-owned file predates this release run and was not modified"]
  summary: "Accepted after verifying the pushed release commit/tag, successful cargo-dist and AUR runs, curated public notes, complete artifact/checksum matrix, extracted release behavior, isolated installer, clean repository, and unchanged independently managed /usr/bin/tandem file."
  evidence: ["Release run: https://github.com/Algorant/tandem/actions/runs/29973001468", "AUR run: https://github.com/Algorant/tandem/actions/runs/29973139232", "Release URL: https://github.com/Algorant/tandem/releases/tag/tandem-v0.6.1", "Release commit: 3eb9309570ed5b8d746e0f9f7ea5414f20526182", "AUR commit: bc4c7b3f4dde220d8b85ebe9d829269fc2543bb2"]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-23T02:05:02Z"
completedAt: "2026-07-23T02:05:15Z"
completion:
  summary: "Released and verified Tandem v0.6.1 from 3eb9309: pushed main and annotated tag, published curated GitHub Release notes and cargo-dist artifacts, validated checksums/archive/isolated installer and released body-edit/cancel behavior, and confirmed tandem-bin AUR 0.6.1 publication."
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md"]
  validation: "161 Rust tests and the full release matrix passed. GitHub Release run 29973001468 succeeded with all four platform archives, installer, per-artifact checksums, and sha256.sum; 605 docs links passed; curated notes include Bug fixes and no installation guidance. Released x86_64 binary passed body-update, cancellation, JSON/Log, and event-privacy smoke. AUR run 29973139232 succeeded; AUR commit bc4c7b3 publishes 0.6.1-1 with checksum 98fda50355f9f175de9c3eb633cb69c34790388a9829a9a0b5c3f1481346bc25. /usr/bin/tandem was not modified."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Cut and independently verify Tandem v0.6.1 after task-165 and task-166 are accepted, integrated, and fully validated.

## Scope and requirements

- Revalidate current `main`, including body update, cancellation, CLI/JSON/TUI behavior, pi-tandem mapping, protocol/docs, hierarchy/progress, events, and existing release surfaces.
- Update Cargo/package/release references consistently to 0.6.1.
- Curate `tandem/GITHUB_RELEASE_NOTES.md` with concise public notes, a dedicated `Bug fixes` section, and no installation guidance.
- Follow `tandem/RELEASE.md`; create an annotated `tandem-v0.6.1` tag from the approved release commit.
- Push `main` and the tag only after local validation and explicit release readiness.
- Verify cargo-dist/GitHub Actions, release artifacts/checksums, extracted binary behavior, installer, curated GitHub Release body, and AUR `tandem-bin` update/checksum.
- Do not install or modify `/usr/bin/tandem`; it intentionally remains independently managed.

## Acceptance criteria

- All Rust, pi-tandem, docs/link, audit, release, and focused behavior validations pass with recorded evidence.
- GitHub Release and artifacts identify v0.6.1 and expose body editing/cancellation behavior correctly.
- Published release notes contain the required Bug fixes section and no installation instructions.
- AUR automation succeeds and points at the correct v0.6.1 asset/checksum.
- Release commit, annotated tag, pushed refs, workflow run IDs, release URL, artifact smoke, and AUR evidence are recorded before acceptance/completion.
- No architecture-refactor implementation or `/usr/bin/tandem` modification occurs.
