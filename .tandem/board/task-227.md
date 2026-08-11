---
id: task-227
type: task
title: "Release Tandem 0.10.1"
state: in-progress
priority: "high"
references: ["task-223", "task-224", "task-225", "task-226", "decision-5"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/RELEASE.md", "justfile"]
tags: ["chore", "release"]
createdAt: "2026-08-11T03:51:23Z"
updatedAt: "2026-08-11T03:51:28Z"
accord:
  status: "claimed"
  assignee: "pi"
  claimedAt: "2026-08-11T03:51:28Z"
  updatedAt: "2026-08-11T03:51:28Z"
assignee: "pi"
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

