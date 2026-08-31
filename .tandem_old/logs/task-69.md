---
id: task-69
type: task
title: "Support global Tandem theme selection from user config"
priority: "high"
relatedFiles: ["tandem/src/tui/theme.rs", "tandem/README.md", "tandem/plan/spec.md"]
tags: ["tui", "theme", "xdg", "config"]
createdAt: "2026-06-30T03:09:11Z"
updatedAt: "2026-06-30T12:24:51Z"
subtasks:
  - id: task-69-1
    title: "Add global user theme selector loading"
    completed: false
  - id: task-69-2
    title: "Document precedence and example config"
    completed: false
  - id: task-69-3
    title: "Validate with tests or smoke run"
    completed: false
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-06-30T03:09:15Z"
  deliveredAt: "2026-06-30T03:18:15Z"
  validation:
    commands: ["Removed workspace-local .tandem/theme.toml so theme selection comes only from ~/.config/tandem/config.toml and ~/.config/tandem/themes/verdigris.toml.", "Confirmed git status clean after cleanup.", "User instructed to reload with just dev for final visual verification."]
  summary: "User verified global Tandem theme config cleanup and requested completion."
  filesChanged: ["tandem/src/tui/theme.rs", "tandem/README.md", "tandem/RELEASE.md", "tandem/plan/spec.md", "README.md", "plan/spec.md", "docs/tui/index.md", "AGENTS.md", "/home/ivan/.dotfiles/tandem/.config/tandem/config.toml", "/home/ivan/.dotfiles/tandem/.config/tandem/themes/verdigris.toml"]
  reviewer: "user"
  updatedAt: "2026-06-30T12:24:47Z"
completedAt: "2026-06-30T12:24:51Z"
completion:
  summary: "Added global Tandem theme selection from user config, documented verification path, removed the workspace-local theme override, and left theme selection centralized in ~/.config/tandem."
  validation: "Accord accepted by user; workspace .tandem/theme.toml removed; ~/.config/tandem/config.toml and ~/.config/tandem/themes/verdigris.toml remain as the only active local theme config; git status clean."
  reviewer: "user"
---

## Description

Make Tandem behave like typical Linux/XDG software by allowing a global user config file under ~/.config/tandem (or $XDG_CONFIG_HOME/tandem) to select the active TUI theme and transparency. Global config should avoid requiring every workspace to mirror .tandem/theme.toml, while preserving workspace override precedence for project-specific choices.
