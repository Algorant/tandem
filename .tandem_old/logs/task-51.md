---
id: task-51
type: task
title: "Add CLI version flag"
priority: "medium"
relatedFiles: ["tandem/src/main.rs", "tandem/Cargo.toml", "tandem/README.md"]
tags: ["cli", "release", "ux"]
createdAt: "2026-06-28T22:39:10Z"
updatedAt: "2026-06-29T01:27:12Z"
subtasks:
  - id: task-51-1
    title: "Add top-level --version and -V handling before subcommand dispatch"
    completed: false
  - id: task-51-2
    title: "Print the Cargo package version via env!(\"CARGO_PKG_VERSION\") or equivalent"
    completed: false
  - id: task-51-3
    title: "Update help/readme/release docs to mention version verification"
    completed: false
  - id: task-51-4
    title: "Add a regression test for version flag output"
    completed: false
  - id: task-51-5
    title: "Run cargo fmt, cargo test, and cargo build"
    completed: false
accord:
  status: "delivered"
  assignee: "task51-version-flag"
  claimedAt: "2026-06-29T00:17:38Z"
  deliveredAt: "2026-06-29T00:17:49Z"
  summary: "Implemented standard Tandem CLI version support. `tandem --version` and `tandem version` now print `tandem <Cargo package version>` from `CARGO_PKG_VERSION`; help text, README install guidance, and release notes now document version verification. Added unit coverage for the version text."
  evidence: ["cd tandem && cargo fmt --check && cargo test (passed: 61 tests)", "cd tandem && cargo run -- --version (prints tandem 0.2.0)", "cd tandem && cargo run -- version (prints tandem 0.2.0)", "git diff --check (passed)", "Committed d2d7e17 Add CLI version command on branch herd-task51-version"]
  filesChanged: ["tandem/src/main.rs", "tandem/README.md", "tandem/RELEASE.md"]
  updatedAt: "2026-06-29T00:17:49Z"
completedAt: "2026-06-29T01:27:12Z"
completion:
  summary: "Accepted as test-verifiable CLI/release work. Implemented `tandem --version` and `tandem version`, updated docs, and validated with cargo tests and focused CLI smoke."
  filesChanged: ["tandem/src/main.rs", "tandem/README.md", "tandem/RELEASE.md"]
  validation: "cargo fmt --check && cargo test passed; version commands print tandem 0.2.0"
---

## Description

Add a standard CLI version flag so users and integrations can verify the installed Tandem binary version. Support  and likely , printing the crate package version from Cargo metadata. Update help text, release/install docs, and tests so future releases can be verified directly.
