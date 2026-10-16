---
title: Configuration
description: Configure TUI themes and Board badges.
---

Tandem loads settings in this order; later values override earlier ones:

1. Built-in `default-dark` and `verdigris` themes.
2. User theme files in `$XDG_CONFIG_HOME/tandem/themes/` (or `~/.config/tandem/themes/`).
3. User settings in `$XDG_CONFIG_HOME/tandem/config.toml` (or `~/.config/tandem/config.toml`).
4. Workspace theme selection and overrides in `.tandem/theme.toml`.
5. Workspace Board display settings in `.tandem/config.toml`.

Use user settings for your usual TUI preference:

```toml
theme = "verdigris"
transparent_background = true
badge_style = "muted"
```

A user theme file may extend a built-in and override selected colors:

```toml
name = "my-dark-theme"
base = "default-dark"

[colors]
accent = "#8ec07c"
```

A project's `.tandem/theme.toml` can select a theme or override its colors. `.tandem/config.toml` controls Board display, including which optional project-tag badges appear. For example:

```toml
[board.badges.tags.release]
label = "release"
tone = "accent"
```

Supported tag tones are `accent`, `success`, `warning`, `error`, `muted`, `orange`, `sand`, and `purple`. Existing standard badges can be hidden with `[board.badges] disabled = ["priority"]`; use `tandem tui` to see the available badges and labels.

The TUI's session board list is temporary and is not stored in user configuration. See the [TUI guide](/tui/) for adding and removing boards in a session.
