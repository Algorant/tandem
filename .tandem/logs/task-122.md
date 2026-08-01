---
id: task-122
type: task
title: "Include parentId in tandem show JSON output"
priority: "medium"
references: ["task-101", "task-103"]
relatedFiles: ["tandem/src/main.rs", "extensions/pi-tandem"]
tags: ["tui", "cli", "relationships", "bug"]
createdAt: "2026-07-10T13:20:09Z"
updatedAt: "2026-07-10T17:23:18Z"
accord:
  status: "accepted"
  assignee: "shep-task-122"
  claimedAt: "2026-07-10T17:20:12Z"
  deliveredAt: "2026-07-10T17:23:04Z"
  deliverables: ["Focused commit 919d0ba6bc3686f1d5c6404c19380367cfd9cef3 on main", "Updated Tandem CLI show JSON and human-readable detail output", "Added Rust regression test and strengthened pi-tandem relationship smoke assertions"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml -- --check — passed", "cargo test --manifest-path tandem/Cargo.toml — 124 passed, 0 failed", "cargo build --manifest-path tandem/Cargo.toml — passed with one pre-existing dead-code warning", "TANDEM_BIN=\"$PWD/tandem/target/debug/tandem\" bun extensions/pi-tandem/tests/relationship-smoke.ts — passed", "Manual child/parent JSON and human-readable show checks — passed", "git diff --check / git show --check — passed; git status --short clean"]
  summary: "Accepted after parent diff review and independent automated validation. The fix is optional-field-safe, covers both JSON and human-readable show output, and pi-tandem receives the corrected CLI response without added parsing logic."
  evidence: ["Worker handoff reported no unexpected dirty files, no blockers, shared main checkout, commit 919d0ba6bc3686f1d5c6404c19380367cfd9cef3", "Parent independently reviewed the complete diff and reran Rust tests and pi-tandem relationship smoke"]
  filesChanged: ["tandem/src/main.rs", "extensions/pi-tandem/tests/relationship-smoke.ts"]
  reviewer: "parent-orchestrator"
  updatedAt: "2026-07-10T17:23:09Z"
completedAt: "2026-07-10T17:23:18Z"
completion:
  summary: "Fixed task detail output so parent-linked tasks expose `parentId` in JSON and `Parent:` in human-readable output, with Rust regression coverage and pi-tandem relationship smoke enforcement."
  filesChanged: ["tandem/src/main.rs", "extensions/pi-tandem/tests/relationship-smoke.ts"]
  validation: "Parent reviewed commit 919d0ba6bc3686f1d5c6404c19380367cfd9cef3 and independently passed cargo fmt --check, all 124 Rust tests, cargo build, pi-tandem relationship smoke, and clean diff/status checks."
  reviewer: "parent-orchestrator"
---

## Description

Bug confirmed with installed Tandem v0.4.3: `.tandem/board/task-102.md` contains `parentId: "task-101"`, but `tandem show task-102 --json` omits the field, so consumers receive no parent relationship even though the TUI and raw document parser can see it.

Scope and acceptance criteria:
- Include `parentId` in `show --json` document details when present.
- Preserve the existing optional-field behavior for documents without a parent.
- Check whether the human-readable show output has the same omission and make relationship presentation consistent where appropriate.
- Add regression coverage for a parent-linked task.
- Confirm pi-tandem `tandem_task action=show` exposes the field through the CLI JSON response.
