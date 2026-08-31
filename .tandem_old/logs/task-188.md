---
id: task-188
type: task
title: "Polish the bare `tandem` CLI landing output"
priority: "medium"
relatedFiles: ["tandem/src/main.rs", "tandem/tests/cli_behavior.rs"]
tags: ["ui", "cli-help", "usability"]
createdAt: "2026-07-28T02:07:51Z"
updatedAt: "2026-07-29T02:26:17Z"
blockers: ["task-146"]
references: ["task-146"]
accord:
  status: "accepted"
  assignee: "worker-task-188-6e715c94"
  claimedAt: "2026-07-29T02:21:59Z"
  deliveredAt: "2026-07-29T02:25:34Z"
  deliverables: ["Commit 484c801", "tandem/src/cli/landing.rs", "CLI integration and real-command tests"]
  validation:
    commands: ["Independent cargo fmt passed", "Independent strict Clippy with all targets/features passed", "Independent cargo test passed: 208 unit + 6 real-command tests", "Direct terminal appearance review passed in Herdr tab 3", "Plain pipe and NO_COLOR contain no ANSI; PTY styling present", "git diff --check passed"]
  constraints: ["No workspace discovery or command behavior changes"]
  summary: "Accepted after code/test review, independent automated validation, direct terminal appearance review, and clean integration to main."
  evidence: ["All 16 top-level commands are covered, including upgrade and tui", "Command-specific help remains separate", "Worker checkout clean"]
  filesChanged: ["tandem/src/cli/landing.rs", "tandem/src/cli/mod.rs", "tandem/tests/cli_behavior.rs"]
  reviewer: "orchestrator"
  updatedAt: "2026-07-29T02:26:10Z"
assignee: "worker-task-188-6e715c94"
completedAt: "2026-07-29T02:26:17Z"
completion:
  summary: "Added and integrated the polished bare tandem CLI landing page as commit 484c801."
  filesChanged: ["tandem/src/cli/landing.rs", "tandem/src/cli/mod.rs", "tandem/tests/cli_behavior.rs"]
  validation: "Direct terminal appearance passed; 208 unit and 6 command tests, focused post-integration test, formatting, strict all-feature Clippy, pipe/NO_COLOR ANSI checks, and diff checks passed."
  reviewer: "orchestrator"
---

## Description

## Objective

Replace the plain usage dump shown by running `tandem` without arguments with a polished, concise command overview.

## Scope

- Add a clear product title and one-line description.
- Group commands by purpose, such as Work, Collaborate, Explore, and Workspace.
- Show aligned command names with short descriptions instead of full flag syntax.
- Direct users to `tandem <command> --help` for detailed usage.
- Use restrained terminal styling when appropriate, while respecting non-TTY output and `NO_COLOR`.
- Preserve command behavior, exit semantics, and script-friendly output.
- Consider a compact workspace-context summary only if it remains fast, reliable, and non-disruptive.

## Acceptance criteria

- Running `tandem` presents a readable, intentionally designed command index.
- Commands are grouped and described consistently, including `upgrade` and `tui`.
- Piped output and `NO_COLOR` contain no unwanted ANSI escapes.
- Existing command parsing and command-specific help remain correct.
- Focused snapshot/text tests cover styled and plain rendering where practical.
- Formatting, full tests, strict Clippy, and real-command CLI tests pass.
- Final appearance receives human terminal review before acceptance.
