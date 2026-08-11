---
id: task-227
type: task
title: "Release Tandem 0.10.1"
state: "validation"
priority: "high"
references: ["task-223", "task-224", "task-225", "task-226", "decision-5"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/RELEASE.md", "justfile"]
tags: ["chore", "release"]
createdAt: "2026-08-11T03:51:23Z"
updatedAt: "2026-08-11T04:01:39Z"
accord:
  status: "blocked"
  assignee: "pi"
  claimedAt: "2026-08-11T03:51:28Z"
  deliveredAt: "2026-08-11T04:01:34Z"
  deliverables: ["Release preparation commit 1e1ad83 pushed to origin/main.", "Annotated tag tandem-v0.10.1 pushed and resolves to 1e1ad83.", "Public non-draft, non-prerelease GitHub Release at https://github.com/Algorant/tandem/releases/tag/tandem-v0.10.1.", "Curated 0.10.1 release notes and 13 non-empty cargo-dist assets.", "Validated x86_64 archive checksum and aggregate sha256.sum entry.", "Primary trytandem.dev installer installed tandem 0.10.1 successfully.", "AUR maintenance failure recorded with workflow and exact upstream message."]
  validation:
    commands: ["`just bench-tui-idle` passed: at 250 Logs, Board 0.75% CPU, Logs 1.00% CPU, input 55.1 ms, reload 290.0 ms.", "Full `just release 0.10.1` local Rust, cargo-dist, docs, Bun audit, web JavaScript, pi-tandem, and diff validation passed before publication.", "Release workflow 31456944108 succeeded.", "Published-release checks passed for notes, draft/prerelease state, and expected assets.", "x86_64 archive SHA-256 check passed and matched sha256.sum.", "`curl -fsSL https://trytandem.dev/install.sh | sh` succeeded; `/home/ivan/.cargo/bin/tandem --version` reports `tandem 0.10.1`.", "AUR workflow 31457094479 reached AUR SSH and failed with: `The AUR is down due to maintenance. We will be back soon.`"]
  constraints: ["Do not reuse or delete tandem-v0.10.1; the GitHub Release and artifacts are published.", "The tandem-bin AUR package remains at 0.8.1-1 until upstream maintenance ends and the workflow is retried."]
  summary: "Published Tandem v0.10.1 from release commit 1e1ad83. Pushed main and annotated tag tandem-v0.10.1; Release workflow 31456944108 succeeded; GitHub Release is public with curated notes and 13 assets; checksums and primary installer passed; installed binary reports tandem 0.10.1. The required downstream AUR workflow 31457094479 failed only because AUR SSH explicitly reports scheduled maintenance before cloning the package repository."
  evidence: ["Release commit 1e1ad831e7548ed0fd00590d4390ef220de9064a.", "Release workflow https://github.com/Algorant/tandem/actions/runs/31456944108.", "AUR workflow https://github.com/Algorant/tandem/actions/runs/31457094479.", "GitHub Release https://github.com/Algorant/tandem/releases/tag/tandem-v0.10.1."]
  filesChanged: [".gitignore", "README.md", "RELEASES.md", "docs/cli/index.md", "docs/reference/index.md", "docs/workspace/index.md", "protocol/plan/spec.md", "tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/README.md", "tandem/RELEASE.md", "tandem/plan/todo.md", ".tandem/logs/task-225.md", ".tandem/logs/task-225-1.md", ".tandem/logs/task-225-2.md", ".tandem/logs/task-225-3.md", ".tandem/logs/task-225-4.md", ".tandem/logs/task-225-5.md", ".tandem/logs/task-225-6.md"]
  reason: "The required tandem-bin AUR workflow 31457094479 failed before package generation because AUR SSH explicitly reports scheduled maintenance. GitHub Release v0.10.1, all 13 assets, checksums, and the primary installer are published and verified. Retry the AUR workflow after upstream maintenance ends; do not recreate or reuse the published tag."
  updatedAt: "2026-08-11T04:01:39Z"
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

