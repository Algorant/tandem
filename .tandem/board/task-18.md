---
id: task-18
type: task
title: "Add first-class user theme discovery and presets"
state: todo
priority: "high"
references: ["task-10"]
relatedFiles: ["tandem-tui/src/tui/theme.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md"]
tags: ["ui", "tui", "theme", "config"]
createdAt: "2026-06-28T00:17:02Z"
updatedAt: "2026-06-28T00:17:02Z"
accord:
  status: "ready"
  deliverables: ["code:tandem-tui/src/tui/theme.rs:user theme discovery and selection", "docs:tandem-tui/README.md:theme install/select examples", "examples:tandem-tui/examples/themes:theme preset examples"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check && cargo test && cargo build", "PTY/manual smoke with ~/.config/tandem/themes custom theme and workspace selector"]
  constraints: ["Do not require users to commit personal themes into project workspaces."]
  updatedAt: "2026-06-28T00:17:02Z"
---

## Description

Make Tandem themes feel first-class for users who want normal `~/.config/tandem` configuration.

Context:
- Current support includes built-in `default-dark`, built-in `verdigris`, and a workspace `.tandem/theme.toml` selector/override.
- The desired model is that users can put theme TOML files in their config directory and select from a small set of presets or a custom theme.

Acceptance direction:
- Implement loading of user TOML themes from `~/.config/tandem/themes/*.toml` according to the documented loading order.
- Provide a small set of documented preset theme examples, including Verdigris and at least one conservative default/dark variant.
- Support selecting a named built-in or user theme from workspace config without copying the full theme into `.tandem/theme.toml`.
- Keep invalid themes non-fatal with clear warnings.
- Update README/spec/todo docs with exact paths and examples.
