---
id: task-95
type: task
title: "Move project tag badge configuration out of theme.toml"
priority: "medium"
references: ["task-79"]
relatedFiles: [".tandem/theme.toml", "tandem/src/tui/theme.rs", "tandem/plan/spec.md", "protocol/plan/spec.md"]
tags: ["tui", "badges", "config", "ux", "protocol"]
createdAt: "2026-07-04T16:41:20Z"
updatedAt: "2026-07-04T18:44:07Z"
subtasks:
  - id: task-95-1
    title: "Decide project/global config file shape for tag badge display semantics"
    completed: false
  - id: task-95-2
    title: "Preserve or document migration behavior from existing theme.toml badge config"
    completed: false
  - id: task-95-3
    title: "Implement parser/loading order changes if in scope"
    completed: false
  - id: task-95-4
    title: "Update specs/docs/examples"
    completed: false
  - id: task-95-5
    title: "Add tests for project/global config badge tag loading and precedence"
    completed: false
accord:
  status: "accepted"
  assignee: "herd:task-95"
  claimedAt: "2026-07-04T17:59:12Z"
  deliveredAt: "2026-07-04T18:36:50Z"
  deliverables: ["New TUI display-config parser applies Board badge semantics from user config and workspace .tandem/config.toml after theme selection.", "Legacy [badges] / [badges.tags.*] parsing is preserved for user theme files and workspace .tandem/theme.toml during migration/backcompat.", "TUI reload fingerprinting watches user config and workspace .tandem/config.toml in addition to theme/board/log files.", "Docs/specs updated to document [board.badges] / [board.badges.tags.<tag>] as the new project badge config home and theme.toml as visual theme-only.", "Local ignored live config moved to .tandem/config.toml; ignored .tandem/theme.toml removed locally because no theme override remains."]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml", "cargo test --manifest-path tandem/Cargo.toml — 117 passed", "git diff --check — clean"]
  summary: "Accepted objective validated config behavior. Implementation moves project/domain Board tag badge semantics to user/workspace display config (`[board.badges]` / `[board.badges.tags.*]`), preserves legacy theme-file badge parsing for migration, and updates reload fingerprinting/docs/specs."
  evidence: ["Same shared main worktree; no commit created.", "Task-95 file subset is separated from concurrent unrelated docs/site changes in the final report.", "Risk/caveat: .tandem/config.toml is ignored local workspace state, so live badge opt-ins are not committed unless a tracked project config mechanism is added later or explicitly allowed."]
  filesChanged: ["tandem/src/tui/theme.rs", "tandem/src/tui.rs", "tandem/plan/spec.md", "tandem/README.md", "tandem/RELEASE.md", "docs/tui/index.md", "docs/guides/theme-tester.md", "protocol/plan/spec.md", "AGENTS.md", "README.md", "plan/spec.md", ".tandem/config.toml", ".tandem/theme.toml"]
  reviewer: "parent/orchestrator"
  updatedAt: "2026-07-04T18:43:45Z"
completedAt: "2026-07-04T18:44:07Z"
completion:
  summary: "Completed project tag badge config migration. Board/TUI project-domain badge semantics now load from user config and workspace `.tandem/config.toml` via `[board.badges]` / `[board.badges.tags.*]`, with legacy theme-file `[badges]` support preserved for migration."
  validation: "Parent/orchestrator reviewed task-95 subset and reran focused tests: workspace display config precedence, legacy workspace theme badge config migration, configured tag badge rendering, disabled IDs/tag labels, and invalid tone warnings all passed. `git diff --check` passed for task-95 subset."
  reviewer: "parent/orchestrator"
---

## Description

Project/domain tag badges are currently configurable through the TUI theme config stack, including workspace `.tandem/theme.toml`. That works mechanically, but it conflates visual theme settings with project display semantics. Design and implement a clearer config home for badge tag opt-ins and related Board/TUI display semantics, likely project-scoped config and/or global user config such as `.tandem/config.toml` / `~/.config/tandem/config.toml`, while preserving compatibility with existing theme.toml behavior during migration if needed.

Context: this repo temporarily uses `.tandem/theme.toml` for tag badge opt-ins like DOCS, SITE, TUI, CLI, SPEC, CI, and BUG. Long-term, theme.toml should remain focused on theme selection/style/color behavior, while project badge semantics should live in config.
