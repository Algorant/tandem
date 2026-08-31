---
id: task-227
type: task
title: "Release Tandem 0.10.1"
priority: "high"
references: ["task-223", "task-224", "task-225", "task-226", "decision-5"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/RELEASE.md", "justfile"]
tags: ["chore", "release"]
createdAt: "2026-08-11T03:51:23Z"
updatedAt: "2026-08-11T12:17:05Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-08-11T03:51:28Z"
  deliveredAt: "2026-08-11T12:16:47Z"
  deliverables: ["Public GitHub Release tandem-v0.10.1 with curated notes and 13 assets.", "Verified checksums and primary installer.", "Global installed binary reports tandem 0.10.1.", "AUR workflow failure recorded as a downstream non-blocking packaging outcome."]
  validation:
    commands: ["Release workflow 31456944108 succeeded.", "Published-release asset and note checks passed.", "x86_64 checksum verification passed.", "Primary installer succeeded and tandem --version reports 0.10.1.", "250-log release benchmark passed at about 1% Logs idle CPU."]
  constraints: ["AUR package remains stale until downstream service access is restored; this does not block the release lifecycle under project rule never:6."]
  summary: "Release v0.10.1 is fully published and verified. Under the user-approved project rule, the downstream AUR maintenance failure is recorded but is not release-blocking."
  evidence: ["https://github.com/Algorant/tandem/releases/tag/tandem-v0.10.1", "https://github.com/Algorant/tandem/actions/runs/31456944108", "AUR failure workflow 31457094479 recorded scheduled maintenance."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "README.md", "tandem/RELEASE.md"]
  reviewer: "user"
  note: "User confirmed AUR failures are non-blocking release-channel outcomes. GitHub Release v0.10.1, notes, assets, checksums, benchmark, and primary installer are verified."
  updatedAt: "2026-08-11T12:16:56Z"
assignee: "pi"
completedAt: "2026-08-11T12:17:05Z"
completion:
  summary: "Released Tandem v0.10.1. Pushed main and annotated tag, published and verified the GitHub Release with 13 assets, validated checksums and the primary installer, and confirmed the installed binary reports 0.10.1. The AUR workflow failure was recorded as a non-blocking downstream packaging outcome under project rule never:6."
  filesChanged: [".gitignore", "README.md", "RELEASES.md", "docs/cli/index.md", "docs/reference/index.md", "docs/workspace/index.md", "protocol/plan/spec.md", "tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/README.md", "tandem/RELEASE.md", "tandem/plan/todo.md"]
  validation: "Release workflow 31456944108 succeeded; public GitHub Release and 13 assets verified; curated notes and checksums passed; primary installer installed tandem 0.10.1; 250-log benchmark passed at about 1% idle CPU; AUR workflow 31457094479 failed only at downstream AUR maintenance and is non-blocking by project policy."
  reviewer: "user"
---

## Description

## Outcome

Prepare, publish, and verify Tandem v0.10.1 as a backward-compatible patch release for the Papercuts TUI surface, coherent TUI keybindings/help, Board chrome cleanup, and Logs performance fix.

## Scope

- Set the Rust package and lockfile version to `0.10.1`.
- Add concise curated `0.10.1` notes to root `RELEASES.md` following decision-5.
- Run the complete release validation in `tandem/RELEASE.md` and `just release`.
- Push `main` and the annotated `tandem-v0.10.1` tag.
- Verify the non-draft GitHub Release, curated notes, expected assets, checksums, and installer.
- Verify the downstream tandem-bin AUR workflow or record a concrete external packaging outage according to project release policy.

## Public positioning

Tandem v0.10.1 makes Papercuts visible in a compact read-only TUI utility inbox, replaces accumulated conflicting hotkeys with a coherent fixed input model and universal help, simplifies duplicated Board chrome, and eliminates Logs-page idle CPU growth at project scale.

## Acceptance criteria

1. `tandem/Cargo.toml` and the Tandem package entry in `Cargo.lock` report `0.10.1`.
2. `RELEASES.md` contains exactly one meaningful `## 0.10.1` section without installation instructions.
3. Full Rust, docs, web JavaScript, pi-tandem, benchmark, release-note, cargo-dist, and diff validation pass.
4. `main` is clean and pushed.
5. Annotated tag `tandem-v0.10.1` is pushed.
6. The GitHub Release exists, is not draft/prerelease, contains curated notes, and has all expected non-empty assets.
7. The published installer reports `tandem 0.10.1`.
8. Release and downstream packaging outcomes are recorded in Tandem.

