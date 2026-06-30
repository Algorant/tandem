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

Board supports quick task creation, state movement, reloads, mouse selection/scrolling, inline preview expansion by clicking the selected row, and opening active task Markdown in `$EDITOR`. Mouse hit regions are intentionally bounded: top tabs switch views, Board state tabs switch states, Board/Logs rows select items, visible footer actions reuse the matching keyboard behavior where safe, wheel events scroll the pane under the pointer, and non-action regions no-op. Drag/drop is out of v0.

## Themes

The TUI includes built-in themes and can load user TOML themes from `~/.config/tandem/themes/` or `$XDG_CONFIG_HOME/tandem/themes/`. Set your normal theme in `~/.config/tandem/config.toml` or `$XDG_CONFIG_HOME/tandem/config.toml`:

```toml
theme = "verdigris"
transparent_background = true
badge_style = "muted"
```

Use workspace `.tandem/theme.toml` only when a project should override the user's global preference.

Set `transparent_background = true` in a user theme, user config, or workspace `.tandem/theme.toml` to let the terminal default/transparent background show through for app and panel fills where practical. The default is `false`, so omitted themes preserve opaque rendering.

Set `badge_style` to control Board priority/status/tag chips. The default, `muted`, keeps the compact badge shape with a softer fill for transparent and image-backed terminals. Other options are:

```toml
badge_style = "muted" # muted chip, default
badge_style = "accent"  # small colored rail plus label
badge_style = "text"    # colored label, no chip fill
badge_style = "ghost"   # outlined/chip text with transparent fill
badge_style = "solid"        # legacy saturated filled block
```

The same key can also be written as `[badges] style = "ghost"` in user themes, user config, or `.tandem/theme.toml`.
