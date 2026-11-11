---
title: Configuration
description: Configure TUI themes and Board badges.
---

tandem loads settings in this order; later values override earlier ones:

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

To open a workspace's Board with some tags hidden, list them in its `.tandem/config.toml`:

```toml
[board]
hide_tags = ["work"]
```

The TUI hides tasks carrying those tags and shows `hiding #work` in the filter bar; choose **Show hidden tags** under `f` to see them for the session. `tandem list`, `tandem search` and `tandem web` are not filtered.

The TUI's session board list is temporary and is not stored in user configuration. See the [TUI guide](/tui/) for adding and removing boards in a session.

The theme files accept these keys:

- root: `theme` (workspace selector), `base`, `builtin`, `extends`, `name`, `transparent_background` (boolean, default `false`), `badge_style` (default `muted`) or `[badges] style`
- `[colors]`: `background`, `panel`, `text`, `muted`, `accent`, `success`, `warning`, `error`, `border`, `selected_bg`, `selected_fg`
- `[priority]`: `critical`, `high`, `medium`, `low`, `none`
- `[aliases]`: named colors, declared before use and reusable by later settings
- `[badges.states]`: workflow state IDs mapped to colors; unlisted IDs keep the muted fallback
- `[badges.tones]`: theme-owned `orange`, `sand` and `purple` tag colors; `beige` is accepted for `sand`
- `[board.badges]`: `disabled` list of built-in badge IDs or tag names; `[board.badges.tags.<tag>]` takes `label` and `tone`
- `[badges.accord]` and `[badges.review]`: legacy per-status overrides still read for existing files

Copy the example presets to start from one:

```text
mkdir -p ~/.config/tandem/themes
cp tandem/examples/themes/default-dark.toml ~/.config/tandem/themes/default-dark.toml
cp tandem/examples/themes/verdigris.toml ~/.config/tandem/themes/verdigris.toml
```

`NO_COLOR=1` or `TANDEM_NO_COLOR=1` forces the terminal/no-color fallback even when a theme is selected. In the built-in Verdigris preset, `in-progress` maps to burnt copper (`#c96f3d`) and `validation` to heather (`#ad8294`); `todo` keeps the muted fallback.
