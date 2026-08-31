---
id: task-163
type: task
title: "Align repository documentation and agent guidance with the final architecture"
priority: "medium"
parentId: "task-146"
blockers: ["task-162"]
references: ["task-145", "decision-7"]
relatedFiles: ["AGENTS.md", "README.md", "plan/refactor_spec.md", "plan/spec.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/README.md", "tandem/plan/spec.md", "extensions/README.md", "extensions/pi-tandem/README.md"]
tags: ["docs", "protocol", "architecture", "refactor"]
createdAt: "2026-07-22T20:43:18Z"
updatedAt: "2026-07-29T00:17:13Z"
accord:
  status: "accepted"
  assignee: "worker-task-163-a57f69a4"
  claimedAt: "2026-07-29T00:11:26Z"
  deliveredAt: "2026-07-29T00:16:54Z"
  deliverables: ["Commit b74bc85", "15 aligned documentation/module-doc files"]
  validation:
    commands: ["Independent cargo fmt check passed", "Independent strict Clippy passed", "Independent cargo test passed: 206 unit + 6 real-command tests", "Independent Bun TypeScript checks passed", "Independent Bun install and Astro site build passed: 15 pages generated and indexed", "git diff --check passed", "Focused stale terminology/path review passed"]
  constraints: ["No production behavior, release, push, or Epic completion"]
  summary: "Accepted after documentation diff review, stale-terminology inspection, independent Rust and TypeScript validation, successful Astro site build, and clean integration."
  evidence: ["Rust diffs are module documentation only", "Documentation identifies protocol/project/app/CLI/TUI ownership and thin CLI-only pi-tandem boundary", "Worker checkout clean"]
  filesChanged: ["AGENTS.md", "README.md", "extensions/README.md", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "plan/refactor_spec.md", "plan/spec.md", "plan/todo.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/README.md", "tandem/plan/spec.md", "tandem/src/cli/mod.rs", "tandem/src/project/mod.rs", "tandem/src/tui/mod.rs"]
  reviewer: "orchestrator"
  updatedAt: "2026-07-29T00:17:06Z"
assignee: "worker-task-163-a57f69a4"
completedAt: "2026-07-29T00:17:13Z"
completion:
  summary: "Aligned repository documentation and agent guidance with the implemented protocol/project/app/CLI/TUI architecture; integrated commit b74bc85."
  filesChanged: ["AGENTS.md", "README.md", "extensions/README.md", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "plan/refactor_spec.md", "plan/spec.md", "plan/todo.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/README.md", "tandem/plan/spec.md", "tandem/src/cli/mod.rs", "tandem/src/project/mod.rs", "tandem/src/tui/mod.rs"]
  validation: "Documentation diff and terminology review passed; Cargo formatting, strict Clippy, 212 tests, Bun TypeScript checks, Astro site build of 15 pages, and diff checks passed."
  reviewer: "orchestrator"
---

## Description

## Objective

Make repository documentation and agent guidance describe the architecture that actually exists after the implementation Tasks, without overstating incomplete or deferred behavior.

## Scope

- Review/update `AGENTS.md`, root/Tandem/protocol/extensions READMEs, relevant specs/todos, and code-level module documentation.
- State clearly that repository `protocol/` Markdown is normative and `tandem/src/protocol/` is its executable Rust implementation.
- Document `project::TandemProject`, shared `app` operations, peer CLI/TUI interfaces, `tui/mod.rs`, protocol 0.2 compatibility, and the thin CLI-only pi-tandem adapter.
- Remove stale references to behavior living in `main.rs`, `tui.rs`, the legacy root `Workspace`, or rejected `persistence` ownership.
- Update task/milestone checklists only where work is actually complete; preserve historical release/spec records.

## Acceptance criteria

- Documentation matches final source paths, dependency direction, protocol behavior, and campaign constraints.
- No document presents proposed leaf filenames/line ranges as immutable protocol requirements.
- Agent guidance prevents duplicate protocol inference and direct pi-tandem parsing/mutation logic.
- Markdown formatting, link checks where available, terminology searches, full tests required by touched generated/documented surfaces, and strict Clippy pass.
- No production behavior change, release, push, or premature Epic completion occurs.

Creating this Task does not authorize starting it.
