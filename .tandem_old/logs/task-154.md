---
id: task-154
type: task
title: "Extract canonical workflow, accord, review, event, and diagnostic semantics"
priority: "high"
parentId: "task-146"
blockers: ["task-153"]
references: ["decision-7"]
relatedFiles: ["plan/refactor_spec.md", "protocol/plan/spec.md", "tandem/src/main.rs", "tandem/src/tui/mod.rs", "tandem/src/protocol/"]
tags: ["protocol", "rust", "architecture", "refactor"]
createdAt: "2026-07-22T20:41:30Z"
updatedAt: "2026-07-26T22:57:12Z"
accord:
  status: "accepted"
  assignee: "worker-task-154-c1384161"
  claimedAt: "2026-07-26T22:13:59Z"
  deliveredAt: "2026-07-26T22:27:18Z"
  deliverables: ["protocol/workflow.rs configurable states and completion semantics", "protocol/accord.rs vocabularies, transitions, state synchronization, event names", "protocol/review.rs metadata vocabulary and completion warnings", "protocol/event.rs canonical and legacy envelope semantics/names", "protocol/diagnostic.rs severity and metadata/workflow/completion checks", "CLI/TUI callers migrated from duplicate lifecycle checks", "source-label adapter preserving exact path diagnostics without path/env handling in protocol"]
  validation:
    commands: ["cargo fmt --check", "179 unit + 4 executable tests passed", "strict Clippy passed", "dependency/duplicate audits passed", "no production std::env/std::path/filesystem access under protocol", "no new protocol suppressions", "one retained Worker rework cycle reviewed"]
  summary: "Extracted canonical workflow/completion, accord, review, event, and diagnostic semantics into protocol modules, with concrete event/path I/O retained outside protocol."
  evidence: ["integration HEAD aad6f14 equals retained Worker HEAD", "git status clean in integration and Worker checkouts", "task-154 accord delivered/state validation", "full tests and strict Clippy independently passed before crash", "protocol production audit excludes env/path/filesystem handling"]
  filesChanged: ["tandem/src/protocol/workflow.rs", "tandem/src/protocol/accord.rs", "tandem/src/protocol/review.rs", "tandem/src/protocol/event.rs", "tandem/src/protocol/diagnostic.rs", "tandem/src/protocol/config.rs", "tandem/src/protocol/document.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/protocol/mod.rs", "tandem/src/main.rs", "tandem/src/tui/mod.rs"]
  reviewer: "parent-orchestrator"
  note: "Post-crash recovery verified the reviewed Worker commits b10956d/aad6f14 are already fast-forward integrated at current refactor/protocol-architecture HEAD aad6f14, the integration tree and retained worktree are clean, and task-154 is correctly delivered in validation. Before the crash, parent independently passed 179 unit + 4 executable tests, strict Clippy, formatting, and the corrected protocol dependency audit."
  updatedAt: "2026-07-26T22:57:02Z"
assignee: "worker-task-154-c1384161"
completedAt: "2026-07-26T22:57:12Z"
completion:
  summary: "Extracted canonical workflow/completion, accord, review, event-envelope, and diagnostic semantics into protocol modules; migrated CLI/TUI callers and kept concrete event/path I/O outside protocol after a reviewed correction."
  filesChanged: ["tandem/src/protocol/workflow.rs", "tandem/src/protocol/accord.rs", "tandem/src/protocol/review.rs", "tandem/src/protocol/event.rs", "tandem/src/protocol/diagnostic.rs", "tandem/src/protocol/config.rs", "tandem/src/protocol/document.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/protocol/mod.rs", "tandem/src/main.rs", "tandem/src/tui/mod.rs"]
  validation: "Parent independently passed formatting, 179 unit tests, 4 executable tests, strict Clippy, and dependency/duplicate audits. Post-Herdr-crash recovery verified integration HEAD aad6f14 matches the clean retained Worker checkout and the delivered task state before acceptance."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Complete the executable-protocol semantic boundary for lifecycle and validation concerns without moving concrete project filesystem operations.

## Scope

- Establish cohesive protocol modules for workflow/completion, accord, review, event envelopes/names, and diagnostics/severity.
- Preserve separation between workflow `state`, `accord`, and `review` metadata.
- Preserve configurable active workflow states, canonical defaults, accepted accord/review vocabularies, completion-to-Logs semantics, fixed warning policy, required event fields/identity, and fail-closed structural diagnostics.
- Keep actor-log path discovery, sequence lookup, JSONL append, raw patching, and board-to-log file movement in the later project boundary.
- Switch callers to canonical protocol values/rules and remove duplicated lifecycle inference.

## Acceptance criteria

- There is one implementation of workflow, completion, accord, review, event-envelope, and diagnostic-category rules.
- Existing protocol 0.2 behavior, event shapes, warning/error severity, CLI output, and TUI diagnostics remain unchanged.
- Unit tests move with semantics and process-level mutation/event tests remain green.
- Formatting, full tests, real-command tests, strict Clippy, and dependency/visibility review pass.
- Temporary lint expectations assigned to these protocol modules are removed.
- No project-I/O extraction, application/interface redesign, release, or push occurs.

Creating this Task does not authorize starting it.
