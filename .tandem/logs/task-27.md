---
id: task-27
type: task
title: "Create first Tandem CLI/TUI release"
priority: "high"
relatedFiles: [".gitignore", "tandem-tui/Cargo.toml", "tandem-tui/Cargo.lock", "tandem-tui/README.md", "tandem-tui/RELEASE.md"]
tags: ["release", "cli", "tui", "tdm"]
createdAt: "2026-06-28T12:51:19Z"
updatedAt: "2026-06-28T13:28:31Z"
subtasks:
  - id: task-27-1
    title: "Confirm release version, tag name, and package scope"
    completed: true
  - id: task-27-2
    title: "Run clean release validation for tandem-tui"
    completed: true
  - id: task-27-3
    title: "Prepare release notes and known limitations"
    completed: true
  - id: task-27-4
    title: "Create/publish the release tag or document publishing blocker"
    completed: true
  - id: task-27-5
    title: "Document install path expected by pi-tandem/global Pi config"
    completed: true
accord:
  status: "accepted"
  assignee: "herd:task27-cli-tui-release"
  claimedAt: "2026-06-28T12:57:29Z"
  deliveredAt: "2026-06-28T13:00:37Z"
  deliverables: ["release:decision:recommended Tandem CLI/TUI release target tandem-tui v0.1.0 / tandem-tui-v0.1.0", "docs:release-notes:tandem-tui/RELEASE.md release notes and known limitations", "docs:install-path:tdm install/lookup instructions for pi-tandem task-15"]
  validation:
    commands: ["git diff --check", "cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build --release"]
  constraints: ["Do not promote global Pi config in this task; only unblock it with a clear tdm release/install target.", "Do not publish artifacts or push tags without an explicit release decision if credentials/settings are unclear."]
  summary: "Prepared the first tdm release target as tandem-tui package v0.1.0 with recommended annotated tag tandem-tui-v0.1.0, release notes, install instructions, and documented tag/publish blocker pending parent approval."
  evidence: ["Validation passed: git diff --check; cd tandem-tui && cargo fmt --check; cd tandem-tui && cargo test; cd tandem-tui && cargo build --release; cd tandem-tui && cargo build --release --locked.", "No tag, push, GitHub release, or artifact was created because release policy/credentials/settings are not yet documented and parent approval is required."]
  filesChanged: [".gitignore", ".tandem/board/task-27.md", "tandem-tui/Cargo.lock", "tandem-tui/README.md", "tandem-tui/RELEASE.md"]
  reviewer: "pi"
  note: "Release prep branch verified and merged; release tag/publish approved by user."
  updatedAt: "2026-06-28T13:28:31Z"
completedAt: "2026-06-28T13:28:31Z"
completion:
  summary: "Prepared and merged the first Tandem CLI/TUI release target for tdm v0.1.0, including release notes, lockfile, and install instructions."
  filesChanged: ["tandem-tui/RELEASE.md", "tandem-tui/README.md", "tandem-tui/Cargo.lock", ".gitignore"]
  validation: "git diff --check; cd tandem-tui && cargo fmt --check; cargo test; cargo build --release --locked"
  reviewer: "pi"
---

## Description

Create the first release of the Tandem CLI/TUI (`tdm`) before promoting pi-tandem into the canonical global Pi config. The global extension/manifest should depend on a real installable or tagged `tdm` release rather than an unreleased workspace binary.

Release scope:
- Decide and document the release version/tag for the current CLI/TUI crate.
- Verify the release build/test path from `tandem-tui/`.
- Prepare release notes that summarize current CLI/TUI capabilities and known limitations.
- Create the git tag/release artifact path appropriate for this repo, or document any blocker if publishing is not yet ready.
- Confirm how downstream Pi config should install or locate `tdm` after the release.

Acceptance direction:
- `tdm` has a clear release tag/version and release notes.
- Release validation has been run from a clean tree.
- Follow-up install instructions are clear enough for task-15 to update the Pi manifest/global config without guessing.
