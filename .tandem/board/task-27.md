---
id: task-27
type: task
title: "Create first Tandem CLI/TUI release"
state: "in-progress"
priority: "high"
relatedFiles: ["tandem-tui/Cargo.toml", "tandem-tui/Cargo.lock", "tandem-tui/README.md", "tandem-tui/plan/spec.md"]
tags: ["release", "cli", "tui", "tdm"]
createdAt: "2026-06-28T12:51:19Z"
updatedAt: "2026-06-28T12:57:29Z"
subtasks:
  - id: task-27-1
    title: "Confirm release version, tag name, and package scope"
    completed: false
  - id: task-27-2
    title: "Run clean release validation for tandem-tui"
    completed: false
  - id: task-27-3
    title: "Prepare release notes and known limitations"
    completed: false
  - id: task-27-4
    title: "Create/publish the release tag or document publishing blocker"
    completed: false
  - id: task-27-5
    title: "Document install path expected by pi-tandem/global Pi config"
    completed: false
accord:
  status: "claimed"
  assignee: "herd:task27-cli-tui-release"
  claimedAt: "2026-06-28T12:57:29Z"
  deliverables: ["release:tag:tagged Tandem CLI/TUI release for tdm", "docs:release-notes:release notes and known limitations", "docs:install-path:tdm install/lookup instructions for pi-tandem task-15"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build --release"]
  constraints: ["Do not promote global Pi config in this task; only unblock it with a clear tdm release/install target.", "Do not publish artifacts or push tags without an explicit release decision if credentials/settings are unclear."]
  updatedAt: "2026-06-28T12:57:29Z"
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
