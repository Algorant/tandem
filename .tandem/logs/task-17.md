---
id: task-17
type: task
title: "Use Verdigris as this repo's default TUI theme"
priority: "medium"
references: ["task-10"]
relatedFiles: [".tandem/theme.toml", "tandem-tui/examples/themes/verdigris.toml", "tandem-tui/src/tui/theme.rs"]
tags: ["ui", "tui", "theme", "repo-config"]
createdAt: "2026-06-28T00:17:02Z"
updatedAt: "2026-06-28T00:24:37Z"
accord:
  status: "accepted"
  assignee: "orchestrator"
  claimedAt: "2026-06-28T00:24:37Z"
  deliveredAt: "2026-06-28T00:24:37Z"
  deliverables: ["config:.tandem/theme.toml:workspace theme selector"]
  validation:
    commands: ["./tandem-tui/target/debug/tdm tui # manual/PTY smoke shows built-in verdigris", "NO_COLOR=1 ./tandem-tui/target/debug/tdm tui # no-color fallback smoke"]
  constraints: ["This is a repository-local preference, not the global Tandem default."]
  summary: "Added workspace .tandem/theme.toml selecting the built-in Verdigris theme as this repo's default."
  evidence: ["Repo-default Verdigris PTY smoke captured theme output; NO_COLOR PTY smoke passed."]
  filesChanged: [".tandem/theme.toml"]
  reviewer: "orchestrator"
  note: "Verified repo-local theme selector loads Verdigris and no-color fallback still exits cleanly."
  updatedAt: "2026-06-28T00:24:37Z"
completedAt: "2026-06-28T00:24:37Z"
completion:
  summary: "Committed repo-local .tandem/theme.toml selecting the built-in Verdigris TUI theme."
  filesChanged: [".tandem/theme.toml"]
  validation: "Repo-default Verdigris PTY smoke; NO_COLOR PTY smoke; git diff --check -- .tandem/theme.toml"
  reviewer: "orchestrator"
---

## Description

Make Verdigris the default Tandem TUI theme for this repository/workspace.

Context:
- Verdigris was implemented in task-10 and the user wants it as the default theme in this Tandem repo for day-to-day use.
- This should be a workspace-level preference, not a global Tandem default.

Acceptance direction:
- Add the minimal committed workspace configuration needed for this repo to load the built-in `verdigris` theme by default.
- Keep the configuration explicit and easy to remove/override.
- Verify `tdm tui` reports/uses the Verdigris base in this repo.
- Preserve `NO_COLOR`/`TANDEM_NO_COLOR` fallback.
