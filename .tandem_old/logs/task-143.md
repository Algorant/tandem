---
id: task-143
type: task
title: "Publish and integration-test the canonical three-tier hierarchy"
priority: "high"
blockers: ["task-140", "task-141", "task-142"]
references: ["decision-7", "task-133"]
relatedFiles: ["README.md", "protocol/README.md", "tandem/README.md", "extensions/README.md", "docs", "site", "justfile", "plan/delegated-task-tree-worker-spec.md"]
tags: ["integration", "docs", "tests", "hierarchy", "visual", "ids"]
createdAt: "2026-07-15T19:45:33Z"
updatedAt: "2026-07-22T04:15:01Z"
parentId: "task-134"
accord:
  status: "accepted"
  assignee: "shep-task-143"
  claimedAt: "2026-07-22T03:59:16Z"
  deliveredAt: "2026-07-22T04:13:46Z"
  deliverables: ["Canonical Epic → global Task → parent-derived leaf Subtask guidance across root, protocol, CLI/TUI, extension, Concepts, and Quickstart documentation.", "Expanded pi-tandem relationship smoke coverage for standalone, Epic-parented, generic-parented, completed-suffix, immutable-reparenting, invalid-depth, and role/ID mismatch cases.", "Task-only delegated campaign and repository handoff assertions.", "Bun site overrides resolving high-severity sharp/svgo audit findings while preserving a frozen lockfile build.", "Subtask task-143-2 implementation complete within the root Task campaign; task-143-1 remained completed historical context."]
  validation:
    commands: ["Parent review: amended diff and all 14 changed files inspected; worktree clean; git show --check and git diff --check passed.", "cargo fmt --manifest-path tandem/Cargo.toml --check passed.", "cargo test --manifest-path tandem/Cargo.toml passed: 154 tests.", "cargo build --release --manifest-path tandem/Cargo.toml passed.", "Focused narrow hierarchy render test passed.", "site: bun install --frozen-lockfile, bun audit --audit-level=high, and bun run check:docs passed; 602 internal links across 15 pages.", "bun --check and pi-tandem smoke.ts, relationship-smoke.ts, and pi-runtime-smoke.ts passed against the release binary.", "Read-only project-local hierarchy checks confirmed task-133 and task-134 expose only global direct Tasks with epic-task relationships.", "User confirmed human visual validation."]
  summary: "Accepted after parent review of amended commit 0309e2f, exact fast-forward integration, independent Rust/Bun/docs/relationship validation, strict real-workspace hierarchy reads, completion of Subtask task-143-2, and explicit user approval of visible hierarchy terminology."
  evidence: ["Commit 0309e2f5f0d39e4fa414bb42f72b39b5f780deb1 supersedes 5a7f751.", "tandem/GITHUB_RELEASE_NOTES.md is byte-for-byte unchanged from main commit 3c1712e.", "Normative stale-language scan is clean; historical v0.5 release prose intentionally excluded."]
  filesChanged: ["README.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/extensions/index.md", "docs/protocol/index.md", "docs/quick-start/index.md", "docs/tui/index.md", "extensions/README.md", "extensions/pi-tandem/tests/relationship-smoke.md", "extensions/pi-tandem/tests/relationship-smoke.ts", "protocol/README.md", "site/bun.lock", "site/package.json", "tandem/README.md"]
  reviewer: "user-and-parent-orchestrator"
  updatedAt: "2026-07-22T04:14:54Z"
completedAt: "2026-07-22T04:15:01Z"
completion:
  summary: "Published and integration-tested the canonical three-tier hierarchy across protocol, CLI/TUI, public docs, and pi-tandem surfaces; integrated commit 0309e2f after aggregate Subtask completion and user visual approval."
  filesChanged: ["README.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/extensions/index.md", "docs/protocol/index.md", "docs/quick-start/index.md", "docs/tui/index.md", "extensions/README.md", "extensions/pi-tandem/tests/relationship-smoke.md", "extensions/pi-tandem/tests/relationship-smoke.ts", "protocol/README.md", "site/bun.lock", "site/package.json", "tandem/README.md"]
  validation: "Parent review; 154 Rust tests; release build; focused narrow hierarchy render; Bun syntax and three pi-tandem smokes; frozen docs install and high-severity audit; 15-page build with 602 link checks; strict task-133/task-134 workspace reads; and explicit user validation all passed."
  reviewer: "user-and-parent-orchestrator"
---

## Description

This is a direct Task of Epic task-134. Integrate all corrected surfaces and ensure no stale public or internal language remains.

Acceptance criteria:
- Update root/tandem/protocol/extensions READMEs and public concepts, CLI, TUI, extension, and quick-start documentation with canonical Epic → global Task → parent-derived Subtask examples.
- Remove claims that direct Epic children are Subtasks, use hierarchical IDs, or receive legacy compatibility.
- Add end-to-end temporary-workspace coverage for standalone Task → Subtask, Epic → global Task → Subtask, generic parent → Task → Subtask, completed suffix allocation, canonical immutable reparenting, invalid nested Epics, invalid Subtask children, and ID/role mismatches.
- Verify task-133 and task-134 expose globally numbered Tasks and no erroneous hierarchical direct children.
- Validate Task-only delegation metadata and the repository side of the delegated-task-tree worker handoff.
- Run full Rust formatting/tests/build, Bun install/audit/smokes, docs build/link checks, and narrow visual fixture validation.
- Complete only after human approval of visible TUI terminology and hierarchy.
