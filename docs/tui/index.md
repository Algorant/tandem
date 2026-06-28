---
title: TUI
description: Using the Tandem terminal user interface.
---

# TUI

`tandem tui` opens the Ratatui-based terminal interface for a Tandem workspace.

## Views

The v0 TUI includes:

- **Board** — active tasks grouped by configured workflow state.
- **Review / Validation** — work that needs validation attention.
- **Logs** — completed task history.
- **Rules** — project coordination rules.
- **Decisions** — decision documents.

## Navigation

Use `1` through `5` to switch top-level views, `j`/`k` or arrow keys to move selection, `h`/`l` for local pane or state navigation, `?` for help, and `q` to quit.

Board supports quick task creation, state movement, reloads, mouse selection/scrolling, and opening active task Markdown in `$EDITOR`.

## Themes

The TUI includes built-in themes and can load user TOML themes from `~/.config/tandem/themes/` or `$XDG_CONFIG_HOME/tandem/themes/`. A workspace can select or override a theme with `.tandem/theme.toml`.
